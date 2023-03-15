use std::rc::Rc;

use miette::{Diagnostic, SourceSpan};
use miette_util::ToMiette;
use thiserror::Error;

use data::string::{comma_separated, separated};
use data::HashSet;
use normalizer::result::EvalError;
use syntax::ast::LookupError;
use syntax::common::*;
use syntax::nf;
use syntax::ust;

use printer::PrintToString;

#[derive(Error, Diagnostic, Debug)]
pub enum TypeError {
    #[error("Wrong number of arguments to {name} provided: got {actual}, expected {expected}")]
    #[diagnostic(code("T-001"))]
    ArgLenMismatch {
        name: String,
        expected: usize,
        actual: usize,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("The following terms are not equal:\n  1: {lhs}\n  2: {rhs}\n")]
    #[diagnostic(code("T-002"))]
    NotEq {
        lhs: String,
        rhs: String,
        #[label("Source of (1)")]
        lhs_span: Option<SourceSpan>,
        #[label("Source of (2)")]
        rhs_span: Option<SourceSpan>,
    },
    #[error("Cannot match on codata type {name}")]
    #[diagnostic(code("T-003"))]
    MatchOnCodata {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot comatch on data type {name}")]
    #[diagnostic(code("T-004"))]
    ComatchOnData {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Invalid pattern match: {msg}")]
    #[diagnostic(code("T-005"))]
    InvalidMatch {
        msg: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Got {actual}, which is not in type {expected}")]
    #[diagnostic(code("T-006"))]
    NotInType {
        expected: Ident,
        actual: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Pattern for {name} is marked as absurd but that could not be proven")]
    #[diagnostic(code("T-007"))]
    PatternIsNotAbsurd {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Pattern for {name} is absurd and must be marked accordingly")]
    #[diagnostic(code("T-008"))]
    PatternIsAbsurd {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Type annotation required for match expression")]
    #[diagnostic(code("T-009"))]
    CannotInferMatch {
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Type annotation required for comatch expression")]
    #[diagnostic(code("T-010"))]
    CannotInferComatch {
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected type constructor application, got {got}")]
    #[diagnostic(code("T-011"))]
    ExpectedTypApp {
        got: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("The impossible happened: {message}")]
    #[diagnostic(code("T-XXX"))]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible {
        message: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    Unify(#[from] UnifyError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Eval(#[from] EvalError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lookup(#[from] LookupError),
}

impl TypeError {
    pub fn not_eq(lhs: Rc<nf::Nf>, rhs: Rc<nf::Nf>) -> Self {
        Self::NotEq {
            lhs: lhs.print_to_string(None),
            rhs: rhs.print_to_string(None),
            lhs_span: lhs.info().span.to_miette(),
            rhs_span: rhs.info().span.to_miette(),
        }
    }

    pub fn invalid_match(
        missing: HashSet<String>,
        undeclared: HashSet<String>,
        duplicate: HashSet<String>,
        info: &ust::Info,
    ) -> Self {
        let mut msgs = Vec::new();

        if !missing.is_empty() {
            msgs.push(format!("missing {}", comma_separated(missing.iter().cloned())));
        }
        if !undeclared.is_empty() {
            msgs.push(format!("undeclared {}", comma_separated(undeclared.iter().cloned())));
        }
        if !duplicate.is_empty() {
            msgs.push(format!("duplicate {}", comma_separated(duplicate.iter().cloned())));
        }

        Self::InvalidMatch { msg: separated("; ", msgs), span: info.span.to_miette() }
    }

    pub fn expected_typ_app(got: Rc<nf::Nf>) -> Self {
        Self::ExpectedTypApp { got: got.print_to_string(None), span: got.info().span.to_miette() }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum UnifyError {
    #[error("{idx} occurs in {exp}")]
    #[diagnostic(code("U-001"))]
    OccursCheckFailed {
        idx: Idx,
        exp: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot unify annotated expression {exp}")]
    #[diagnostic(code("U-002"))]
    UnsupportedAnnotation {
        exp: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot automatically decide whether {lhs} and {rhs} unify")]
    #[diagnostic(code("U-003"))]
    CannotDecide {
        lhs: String,
        rhs: String,
        #[label]
        lhs_span: Option<SourceSpan>,
        #[label]
        rhs_span: Option<SourceSpan>,
    },
}

impl UnifyError {
    pub fn occurs_check_failed(idx: Idx, exp: Rc<ust::Exp>) -> Self {
        Self::OccursCheckFailed {
            idx,
            exp: exp.print_to_string(None),
            span: exp.span().to_miette(),
        }
    }

    pub fn unsupported_annotation(exp: Rc<ust::Exp>) -> Self {
        Self::UnsupportedAnnotation { exp: exp.print_to_string(None), span: exp.span().to_miette() }
    }

    pub fn cannot_decide(lhs: Rc<ust::Exp>, rhs: Rc<ust::Exp>) -> Self {
        Self::CannotDecide {
            lhs: lhs.print_to_string(None),
            rhs: rhs.print_to_string(None),
            lhs_span: lhs.span().to_miette(),
            rhs_span: rhs.span().to_miette(),
        }
    }
}
