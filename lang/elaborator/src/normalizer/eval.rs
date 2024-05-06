use std::rc::Rc;

use syntax::ctx::{BindContext, Context};
use syntax::generic::*;
use tracer::trace;

use crate::normalizer::env::*;
use crate::normalizer::val::{self, Closure, Neu, Val};

use crate::result::*;

pub trait Eval {
    type Val;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError>;
}

pub trait Apply {
    fn apply(self, prg: &Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError>;
}

impl Eval for Exp {
    type Val = Rc<Val>;

    #[trace("{:P} |- {:P} ▷ {return:P}", env, self, std::convert::identity)]
    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        match self {
            Exp::Variable(e) => e.eval(prg, env),
            Exp::TypCtor(e) => e.eval(prg, env),
            Exp::Call(e) => e.eval(prg, env),
            Exp::DotCall(e) => e.eval(prg, env),
            Exp::Anno(e) => e.eval(prg, env),
            Exp::TypeUniv(e) => e.eval(prg, env),
            Exp::LocalMatch(e) => e.eval(prg, env),
            Exp::LocalComatch(e) => e.eval(prg, env),
            Exp::Hole(e) => e.eval(prg, env),
        }
    }
}

impl Eval for Variable {
    type Val = Rc<Val>;

    fn eval(&self, _prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Variable { idx, .. } = self;
        Ok(env.lookup(*idx))
    }
}

impl Eval for TypCtor {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let TypCtor { span, name, args } = self;
        Ok(Rc::new(Val::TypCtor { span: *span, name: name.clone(), args: args.eval(prg, env)? }))
    }
}

impl Eval for Call {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Call { span, name, kind, args, .. } = self;
        Ok(Rc::new(Val::Ctor {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: args.eval(prg, env)?,
        }))
    }
}

impl Eval for DotCall {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let DotCall { span, exp, name, args, .. } = self;
        let exp = exp.eval(prg, env)?;
        let args = args.eval(prg, env)?;
        match (*exp).clone() {
            Val::Ctor { name: ctor_name, kind: _, args: ctor_args, span } => {
                let type_decl = prg.decls.type_decl_for_member(&ctor_name, span)?;
                match type_decl {
                    DataCodata::Data(_) => {
                        let Def { body, .. } = prg.decls.def(name, None)?;
                        let body =
                            Env::empty().bind_iter(args.iter(), |env| body.eval(prg, env))?;
                        beta_match(prg, body, &ctor_name, &ctor_args)
                    }
                    DataCodata::Codata(_) => {
                        let Codef { body, .. } = prg.decls.codef(&ctor_name, None)?;
                        let body =
                            Env::empty().bind_iter(ctor_args.iter(), |env| body.eval(prg, env))?;
                        beta_comatch(prg, body, name, &args)
                    }
                }
            }
            Val::Comatch { body, .. } => beta_comatch(prg, body, name, &args),
            Val::Neu { exp } => Ok(Rc::new(Val::Neu {
                exp: Neu::Dtor { span: *span, exp: Rc::new(exp), name: name.to_owned(), args },
            })),
            _ => unreachable!(),
        }
    }
}

impl Eval for Anno {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Anno { exp, .. } = self;
        exp.eval(prg, env)
    }
}

impl Eval for TypeUniv {
    type Val = Rc<Val>;

    fn eval(&self, _prg: &Prg, _env: &mut Env) -> Result<Self::Val, TypeError> {
        let TypeUniv { span } = self;
        Ok(Rc::new(Val::TypeUniv { span: *span }))
    }
}

impl Eval for LocalMatch {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let LocalMatch { name: match_name, on_exp, body, .. } = self;
        let on_exp = on_exp.eval(prg, env)?;
        let body = body.eval(prg, env)?;
        match (*on_exp).clone() {
            Val::Ctor { name: ctor_name, args, .. } => beta_match(prg, body, &ctor_name, &args),
            Val::Neu { exp } => Ok(Rc::new(Val::Neu {
                exp: Neu::Match {
                    span: None,
                    name: match_name.to_owned(),
                    on_exp: Rc::new(exp),
                    body,
                },
            })),
            _ => unreachable!(),
        }
    }
}

impl Eval for LocalComatch {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, body, .. } = self;
        Ok(Rc::new(Val::Comatch {
            span: *span,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.eval(prg, env)?,
        }))
    }
}

impl Eval for Hole {
    type Val = Rc<Val>;

    fn eval(&self, _prg: &Prg, _env: &mut Env) -> Result<Self::Val, TypeError> {
        let Hole { span, .. } = self;
        Ok(Rc::new(Val::Neu { exp: Neu::Hole { span: *span } }))
    }
}

#[trace("{}(...).match {:P} ▷β {return:P}", ctor_name, body, std::convert::identity)]
fn beta_match(
    prg: &Prg,
    body: val::Match,
    ctor_name: &str,
    args: &[Rc<Val>],
) -> Result<Rc<Val>, TypeError> {
    let case = body.clone().cases.into_iter().find(|case| case.name == ctor_name).unwrap();
    let val::Case { body, .. } = case;
    let body = body.unwrap();
    body.apply(prg, args)
}

#[trace("comatch {:P}.{}(...) ▷β {return:P}", body, dtor_name, std::convert::identity)]
fn beta_comatch(
    prg: &Prg,
    body: val::Match,
    dtor_name: &str,
    args: &[Rc<Val>],
) -> Result<Rc<Val>, TypeError> {
    let cocase = body.clone().cases.into_iter().find(|cocase| cocase.name == dtor_name).unwrap();
    let val::Case { body, .. } = cocase;
    let body = body.unwrap();
    body.apply(prg, args)
}

impl Eval for Match {
    type Val = val::Match;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Match { span, cases, omit_absurd } = self;

        Ok(val::Match { span: *span, cases: cases.eval(prg, env)?, omit_absurd: *omit_absurd })
    }
}

impl Eval for Case {
    type Val = val::Case;

    fn eval(&self, _prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Case { span, name, params, body } = self;

        let body = body.as_ref().map(|body| Closure {
            body: body.clone(),
            n_args: params.len(),
            env: env.clone(),
        });

        Ok(val::Case { span: *span, name: name.clone(), params: params.clone(), body })
    }
}

impl Eval for Args {
    type Val = Vec<Rc<Val>>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.args.eval(prg, env)
    }
}

impl Apply for Closure {
    fn apply(mut self, prg: &Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError> {
        self.env.bind_iter(args.iter(), |env| self.body.eval(prg, env))
    }
}

impl<T: Eval> Eval for Vec<T> {
    type Val = Vec<T::Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.iter().map(|x| x.eval(prg, env)).collect()
    }
}

impl Eval for Rc<Exp> {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        (**self).eval(prg, env)
    }
}