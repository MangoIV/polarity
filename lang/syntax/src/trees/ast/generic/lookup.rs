use std::fmt;
use std::rc::Rc;

use miette::{Diagnostic, SourceSpan};
use miette_util::ToMiette;
use thiserror::Error;

use data::string::comma_separated;

use crate::common::*;

use super::def::*;
use super::lookup_table;

#[derive(Error, Diagnostic, Debug)]
pub enum LookupError {
    #[error("Undefined top-level declaration {name}")]
    UndefinedDeclaration {
        name: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a {expected}, but it is a {actual}")]
    InvalidDeclarationKind {
        name: String,
        expected: AnyOf<DeclKind>,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Missing type declaration for {name}")]
    MissingTypeDeclaration {
        name: String,
        #[label]
        span: Option<SourceSpan>,
    },
}

impl<P: Phase> Decls<P> {
    pub fn empty() -> Self {
        Self { map: data::HashMap::default(), lookup_table: Default::default() }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<'_, P>> {
        self.lookup_table.iter().map(|item| match item {
            lookup_table::Item::Type(type_decl) => match &self.map[&type_decl.name] {
                Decl::Data(data) => Item::Data(data),
                Decl::Codata(codata) => Item::Codata(codata),
                _ => unreachable!(),
            },
            lookup_table::Item::Def(def_decl) => match &self.map[&def_decl.name] {
                Decl::Def(def) => Item::Def(def),
                Decl::Codef(codef) => Item::Codef(codef),
                _ => unreachable!(),
            },
        })
    }

    pub fn type_decl_for_member(
        &self,
        name: &Ident,
        span: Option<codespan::Span>,
    ) -> Result<Type<'_, P>, LookupError> {
        let type_decl = self
            .lookup_table
            .type_decl_for_xtor(name)
            .or_else(|| self.lookup_table.type_decl_for_xdef(name))
            .ok_or_else(|| LookupError::MissingTypeDeclaration {
                name: name.to_owned(),
                span: span.to_miette(),
            })?;
        self.typ(&type_decl.name, None)
    }

    pub fn data_for_ctor(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Data<P>, LookupError> {
        self.ctor(name, span)?;
        let type_decl = self.lookup_table.type_decl_for_xtor(name).ok_or_else(|| {
            LookupError::MissingTypeDeclaration { name: name.to_owned(), span: span.to_miette() }
        })?;
        self.data(&type_decl.name, None)
    }

    pub fn codata_for_dtor(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Codata<P>, LookupError> {
        self.dtor(name, span)?;
        let type_decl = self.lookup_table.type_decl_for_xtor(name).ok_or_else(|| {
            LookupError::MissingTypeDeclaration { name: name.to_owned(), span: span.to_miette() }
        })?;
        self.codata(&type_decl.name, None)
    }

    pub fn xtors_for_type(&self, name: &str) -> Vec<Ident> {
        self.lookup_table.xtors_for_type(name)
    }

    pub fn xdefs_for_type(&self, name: &str) -> Vec<Ident> {
        self.lookup_table.xdefs_for_type(name)
    }

    pub fn typ(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<Type<'_, P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Data(data) => Ok(Type::Data(data)),
            Decl::Codata(codata) => Ok(Type::Codata(codata)),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: AnyOf(vec![DeclKind::Data, DeclKind::Codata]),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn data(&self, name: &str, span: Option<codespan::Span>) -> Result<&Data<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Data(data) => Ok(data),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: DeclKind::Data.into(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn codata(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Codata<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Codata(codata) => Ok(codata),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: DeclKind::Codata.into(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn def(&self, name: &str, span: Option<codespan::Span>) -> Result<&Def<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Def(def) => Ok(def),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: DeclKind::Def.into(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn codef(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Codef<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Codef(codef) => Ok(codef),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: DeclKind::Codef.into(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn ctor(&self, name: &str, span: Option<codespan::Span>) -> Result<&Ctor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Ctor(ctor) => Ok(ctor),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: DeclKind::Ctor.into(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn dtor(&self, name: &str, span: Option<codespan::Span>) -> Result<&Dtor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Dtor(dtor) => Ok(dtor),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: DeclKind::Dtor.into(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn ctor_or_codef(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<Ctor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Ctor(ctor) => Ok(ctor.clone()),
            Decl::Codef(codef) => Ok(codef.to_ctor()),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: AnyOf(vec![DeclKind::Ctor, DeclKind::Codef]),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn dtor_or_def(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<Dtor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Dtor(dtor) => Ok(dtor.clone()),
            Decl::Def(def) => Ok(def.to_dtor()),
            other => Err(LookupError::InvalidDeclarationKind {
                name: name.to_owned(),
                expected: AnyOf(vec![DeclKind::Dtor, DeclKind::Def]),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    fn decl(&self, name: &str, span: Option<codespan::Span>) -> Result<&Decl<P>, LookupError> {
        self.map.get(name).ok_or_else(|| LookupError::UndefinedDeclaration {
            name: name.to_owned(),
            span: span.to_miette(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Item<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
    Def(&'a Def<P>),
    Codef(&'a Codef<P>),
}

impl<'a, P: Phase> Item<'a, P> {
    pub fn hidden(&self) -> bool {
        match self {
            Item::Data(data) => data.hidden,
            Item::Codata(codata) => codata.hidden,
            Item::Def(def) => def.hidden,
            Item::Codef(codef) => codef.hidden,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
}

impl<'a, P: Phase> Type<'a, P> {
    pub fn name(&self) -> &Ident {
        match self {
            Type::Data(data) => &data.name,
            Type::Codata(codata) => &codata.name,
        }
    }

    pub fn typ(&self) -> Rc<TypAbs<P>> {
        match self {
            Type::Data(data) => data.typ.clone(),
            Type::Codata(codata) => codata.typ.clone(),
        }
    }
}

#[derive(Debug)]
pub struct AnyOf<T>(Vec<T>);

impl<T> From<T> for AnyOf<T> {
    fn from(x: T) -> Self {
        AnyOf(vec![x])
    }
}

impl<T: fmt::Display> fmt::Display for AnyOf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            Ok(())
        } else if self.0.len() == 1 {
            write!(f, "{}", self.0[0])
        } else {
            write!(
                f,
                "{} or {}",
                comma_separated(self.0[..self.0.len() - 1].iter().map(|x| x.to_string())),
                self.0[self.0.len() - 1]
            )
        }
    }
}
