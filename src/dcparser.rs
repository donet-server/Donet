// DONET SOFTWARE
// Copyright (c) 2023, Donet Authors.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3.
// You should have received a copy of this license along
// with this source code in a file named "LICENSE."
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program; if not, write to the Free Software Foundation,
// Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

// The following suppress linting warnings, which are okay to ignore
// as they go off in the parser grammar definitions, which we are writing
// just as the plex crate readme says we should, so everything is okay.
#![allow(clippy::type_complexity, clippy::redundant_field_names, clippy::ptr_arg)]
#![allow(clippy::redundant_closure_call, clippy::enum_variant_names)]

use crate::dclexer::DCToken::*;
use crate::dclexer::{DCToken, Span};
use plex::parser;

mod ast {
    // In this module we store all the structures and enums
    // that make up the final generated abstract syntax tree.
    use super::{DCToken, Span};
    use std::ops::Range;
    pub type IdentifierString = String; // type alias

    #[derive(Debug)]
    pub struct DCFile {
        pub type_decl: Vec<TypeDecl>,
    }

    #[derive(Debug)]
    pub struct TypeDecl {
        pub span: Span,
        pub node: TypeDecl_,
    }

    #[derive(Debug)]
    pub enum TypeDecl_ {
        KeywordType(KeywordType),
        StructType(StructType),
        DistributedClassType(DistributedClassType),
        DCImport(DCImport),
        TypeDefinition(TypeDefinition),
    }

    #[derive(Debug)]
    pub struct KeywordType {
        pub span: Span,
        pub node: KeywordType_,
    }

    #[derive(Debug)]
    pub enum KeywordType_ {
        KeywordType(IdentifierString),
        KeywordList(Vec<IdentifierString>),
    }

    #[derive(Debug)]
    pub struct StructType {
        pub span: Span,
        pub identifier: IdentifierString,
        pub parameters: Vec<ParameterField>,
    }

    #[derive(Debug)]
    pub struct DistributedClassType {
        pub span: Span,
        pub identifier: IdentifierString,
        pub field_declarations: Vec<FieldDecl>,
    }

    #[derive(Debug)]
    pub struct DCImport {
        pub span: Span,
        pub module: Vec<String>, // python filename, or module(s)
        pub module_views: Vec<String>,
        pub class: IdentifierString,
        pub class_views: Vec<String>, // AI, UD, OV ...
    }

    #[derive(Debug)]
    pub struct TypeDefinition {
        pub span: Span,
        pub dc_type: DataType,
        pub alias: IdentifierString,
    }

    #[derive(Debug)]
    pub struct FieldDecl {
        pub span: Span,
        pub node: FieldDecl_,
    }

    #[derive(Debug)]
    pub enum FieldDecl_ {
        MolecularField(MolecularField),
        AtomicField(AtomicField),
        ParameterField(ParameterField),
    }

    #[derive(Debug)]
    pub struct MolecularField {
        pub identifier: IdentifierString,
        pub field_type: FieldType,
    }

    #[derive(Debug)]
    pub enum FieldType {
        Atomic(AtomicField),
        Parameter(ParameterField),
    }

    #[derive(Debug)]
    pub struct AtomicField {
        pub identifier: IdentifierString,
        pub parameters: Vec<Parameter>,
        pub keyword_list: Option<KeywordType>,
    }

    #[derive(Debug)]
    pub struct ParameterField {
        pub parameter: Parameter,
        pub keyword_list: Option<KeywordType>,
    }

    #[derive(Debug)]
    pub enum Parameter {
        Char(CharParameter),
        Int(IntParameter),
        Float(FloatParameter),
        Sized(SizedParameter),
        Struct(StructParameter),
        Array(ArrayParameter),
    }

    #[derive(Debug)]
    pub struct CharParameter {
        pub char_type: Option<IdentifierString>,
        pub char_literal: Option<char>,
    }

    #[derive(Debug)]
    pub struct IntParameter {
        pub identifier: Option<IdentifierString>,
        pub int_type: Option<IdentifierString>,
        pub int_range: Option<Range<i64>>,
        pub int_transform: Option<IntTransform>,
        pub int_constant: Option<i64>,
    }

    #[derive(Debug)]
    pub struct FloatParameter {
        pub identifier: Option<IdentifierString>,
        pub float_type: Option<IdentifierString>,
        pub float_range: Option<Range<f64>>,
        pub float_transform: Option<FloatTransform>,
        pub float_constant: Option<f64>,
    }

    #[derive(Debug)]
    pub struct SizedParameter {
        pub sized_type: Option<IdentifierString>,
        pub size_constraint: Option<i64>,
        pub identifier: Option<IdentifierString>,
        pub string_literal: Option<String>,
    }

    #[derive(Debug)]
    pub struct StructParameter {
        pub identifier1: IdentifierString,
        pub identifier2: Option<IdentifierString>,
    }

    #[derive(Debug)]
    pub struct ArrayParameter {
        pub data_type: DataType,
        pub identifier: Option<IdentifierString>,
        pub array_range: Range<i64>,
    }

    #[derive(Debug)]
    pub struct DataType {
        pub base_type: BaseType,
        pub identifier: Option<String>, // used for IntType (unsigned/signed + bits)
    }

    #[rustfmt::skip]
    #[derive(Debug)]
    pub enum BaseType {
        CharType, IntType, FloatType,
        StringType, BlobType, StructType,
    }

    #[derive(Debug)]
    pub enum IntTransform {
        OperatorIntLiteral { operator: DCToken, int_literal: i32 },
        ParenthesizedIntTransform(Box<IntTransform>),
    }

    #[derive(Debug)]
    pub enum FloatTransform {
        OperatorFloatLiteral { operator: DCToken, float_literal: f32 },
        ParenthesizedFloatTransform(Box<FloatTransform>),
    }
}

// Plex macro to start defining our grammar
parser! {
    fn parse_(DCToken, Span);

    // Instruct parser how to combine two spans
    (a, b) {
        Span {
            min: a.min,
            max: b.max,
            line: a.line, // only keep a's line number
        }
    }

    // DC File (root production of the grammar)
    dc_file: ast::DCFile {
        type_declarations[tds] => ast::DCFile { type_decl: tds },
    }

    // Collect all our Type Declarations into a vector for the DCFile.
    type_declarations: Vec<ast::TypeDecl> {
        => vec![],
        type_declarations[mut td_vec] type_decl[next_td] => {
            td_vec.push(next_td);
            td_vec
        }
    }
    type_decl: ast::TypeDecl {
        keyword_type[k] => ast::TypeDecl {
            span: span!(),
            node: ast::TypeDecl_::KeywordType(k),
        },
        struct_type[s] => ast::TypeDecl {
            span: span!(),
            node: ast::TypeDecl_::StructType(s),
        },
        distributed_class_type[dc] => ast::TypeDecl {
            span: span!(),
            node: ast::TypeDecl_::DistributedClassType(dc),
        },
        dc_import[dci] => ast::TypeDecl {
            span: span!(),
            node: ast::TypeDecl_::DCImport(dci),
        },
        type_definition[td] => ast::TypeDecl {
            span: span!(),
            node: ast::TypeDecl_::TypeDefinition(td),
        },
    }

    keyword_type: ast::KeywordType {
        KeywordType Identifier(id) Semicolon => ast::KeywordType {
            span: span!(),
            node: ast::KeywordType_::KeywordType(id),
        }
    }

    struct_type: ast::StructType {
        StructType Identifier(id) OpenBraces parameters[ps]
        CloseBraces Semicolon => ast::StructType {
            span: span!(),
            identifier: id,
            parameters: ps,
        }
    }

    distributed_class_type: ast::DistributedClassType {

    }

    dc_import: ast::DCImport {
        py_mod[(m, ms)] dc_imp[(c, cs)] => ast::DCImport {
            span: span!(),
            module: vec![m],
            module_views: ms,
            class: c,
            class_views: cs,
        },
        // FIXME: 'reduce_11' never used; dead code warning.
        nested_py_mod[(nm, ms)] dc_imp[(c, cs)] => ast::DCImport {
            span: span!(),
            module: nm,
            module_views: ms,
            class: c,
            class_views: cs,
        },
    }

    type_definition: ast::TypeDefinition {
        TypeDefinition CharType Identifier(alias) Semicolon => ast::TypeDefinition {
            span: span!(),
            dc_type: ast::DataType {
                base_type: ast::BaseType::CharType,
                identifier: None,
            },
            alias: alias,
        },
        TypeDefinition IntType(int_id) Identifier(alias) Semicolon => ast::TypeDefinition {
            span: span!(),
            dc_type: ast::DataType {
                base_type: ast::BaseType::IntType,
                identifier: Some(int_id), // unsigned/signed + bits
            },
            alias: alias,
        },
        TypeDefinition FloatType Identifier(alias) Semicolon => ast::TypeDefinition {
            span: span!(),
            dc_type: ast::DataType {
                base_type: ast::BaseType::FloatType,
                identifier: None,
            },
            alias: alias,
        },
        TypeDefinition StringType Identifier(alias) Semicolon => ast::TypeDefinition {
            span: span!(),
            dc_type: ast::DataType {
                base_type: ast::BaseType::StringType,
                identifier: None,
            },
            alias: alias,
        },
        TypeDefinition BlobType Identifier(alias) Semicolon => ast::TypeDefinition {
            span: span!(),
            dc_type: ast::DataType {
                base_type: ast::BaseType::BlobType,
                identifier: None,
            },
            alias: alias,
        },
    }

    // e.g. "from views ..."
    py_mod: (String, Vec<String>) {
        From import_with_suffix[(m, ms)] => (m, ms),
    }

    // e.g. "from game.views.Donut/AI ..."
    nested_py_mod: (Vec<String>, Vec<String>) {
        From nested_modules_with_suffix[(nm, ms)] => (nm, ms),
    }

    // e.g. "import DistributedDonut/AI/OV"
    dc_imp: (String, Vec<String>) {
        Import import_with_suffix[(c, cs)] => (c, cs),
        // FIXME: Allow class imports without suffixes (and avoid shift-reduce conflict)
        //Import Identifier(c) => (c, vec![]),
    }

    import_with_suffix: (String, Vec<String>) {
        // "from views/AI/OV import DistributedDonut/AI/OV"
        // "from my_views/AI/OV import DistributedDonut/AI/OV"
        Identifier(i) view_suffixes[is] => (i, is),
        Module(i) view_suffixes[is] => (i, is),
    }

    nested_modules_with_suffix: (Vec<String>, Vec<String>) {
        nested_py_modules[nm] view_suffixes[ms] => (nm, ms),
    }

    // Bundles module names in 'from' statements, e.g. "myviews.Donut".
    nested_py_modules: Vec<String> {
        => vec![],
        nested_py_modules[mut nm] py_module[m] => {
            nm.push(m);
            nm
        },
        nested_py_modules[mut nm] py_module[m] => {
            nm.push(m);
            nm
        }
        // FIXME: Handle 1 or more modules without shift-reduce conflict.
        // py_module[m] => vec![m],
    }

    // NOTE: Module names may be lexed as identifiers or module tokens.
    py_module: String {
        Period Identifier(m) => m,
        Period Module(m) => m,
    }

    // Bundle up all views of a dclass/module to be imported, into a vector
    // of strings, each corresponding to a view suffix. (AI, UD, OV..)
    view_suffixes: Vec<String> {
        => vec![],
        view_suffixes[mut vs] view_suffix[s] => {
            vs.push(s);
            vs
        }
        // FIXME: Handle 1 or more suffixes without shift-reduce conflict.
        // view_suffix[s] => vec![s],
    }

    // Matches '/AI' '/OV' from, example, "DistributedDonut/AI/OV"
    view_suffix: String {
        ForwardSlash Identifier(s) => s
    }

    // The 'parameters' production is made up of the current parameters
    // plus a new parameter following, and returns a vector of all
    // parameters parsed so far. This bundles them all up for other productions.
    parameters: Vec<ast::ParameterField> {
        => vec![],
        parameters[mut ps] parameter[p] => {
            ps.push(p);
            ps
        }
    }

    parameter: ast::ParameterField {

    }

}

// This is the interface to our parser; Provides an iterator.
pub fn parse<I: Iterator<Item = (DCToken, Span)>>(
    i: I,
) -> Result<ast::DCFile, (Option<(DCToken, Span)>, &'static str)> {
    parse_(i)
}

#[cfg(test)]
mod unit_testing {
    //use super::*;
}
