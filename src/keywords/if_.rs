use super::CompilationResult;
use super::{Validate, Validators};
use crate::compilation::compile_validators;
use crate::compilation::CompilationContext;
use crate::error::{no_error, ErrorIterator};
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct IfThenValidator {
    schema: Validators,
    then_schema: Validators,
}

impl IfThenValidator {
    pub(crate) fn compile(
        schema: &Value,
        then_schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(IfThenValidator {
            schema: compile_validators(schema, context)?,
            then_schema: compile_validators(then_schema, context)?,
        }))
    }
}

impl Validate for IfThenValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .then_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }
    fn name(&self) -> String {
        format!("<if-then: {:?} {:?}>", self.schema, self.then_schema)
    }
}

pub struct IfElseValidator {
    schema: Validators,
    else_schema: Validators,
}

impl<'a> IfElseValidator {
    pub(crate) fn compile(
        schema: &'a Value,
        else_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(IfElseValidator {
            schema: compile_validators(schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

impl Validate for IfElseValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .any(|validator| !validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .else_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }
    fn name(&self) -> String {
        format!("<if-else: {:?} {:?}>", self.schema, self.else_schema)
    }
}

pub struct IfThenElseValidator {
    schema: Validators,
    then_schema: Validators,
    else_schema: Validators,
}

impl IfThenElseValidator {
    pub(crate) fn compile(
        schema: &Value,
        then_schema: &Value,
        else_schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(IfThenElseValidator {
            schema: compile_validators(schema, context)?,
            then_schema: compile_validators(then_schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

impl Validate for IfThenElseValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .then_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            Box::new(errors.into_iter())
        } else {
            let errors: Vec<_> = self
                .else_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            Box::new(errors.into_iter())
        }
    }
    fn name(&self) -> String {
        format!(
            "<if-then-else: {:?} {:?} {:?}>",
            self.schema, self.then_schema, self.else_schema
        )
    }
}

pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    let then = parent.get("then");
    let else_ = parent.get("else");
    match (then, else_) {
        (Some(then_schema), Some(else_schema)) => Some(IfThenElseValidator::compile(
            schema,
            then_schema,
            else_schema,
            context,
        )),
        (None, Some(else_schema)) => Some(IfElseValidator::compile(schema, else_schema, context)),
        (Some(then_schema), None) => Some(IfThenValidator::compile(schema, then_schema, context)),
        (None, None) => None,
    }
}
