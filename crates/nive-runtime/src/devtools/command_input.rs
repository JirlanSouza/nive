use std::collections::BTreeMap;

use super::{helpers::label_from_field_name, types::DevtoolFieldSchema};

pub struct DevtoolInputValues<'a> {
    values: Option<&'a BTreeMap<String, String>>,
}

pub trait DevtoolOperationContext: Sized {
    fn devtool_fields() -> Vec<DevtoolFieldSchema>;
    fn devtool_build(inputs: &DevtoolInputValues<'_>) -> Result<Self, String>;
}

pub trait DevtoolInputField: Sized {
    fn devtool_schema(name: &'static str) -> DevtoolFieldSchema;
    fn devtool_build(name: &'static str, inputs: &DevtoolInputValues<'_>) -> Result<Self, String>;
}

impl<'a> DevtoolInputValues<'a> {
    pub fn new(values: &'a BTreeMap<String, String>) -> Self {
        Self {
            values: Some(values),
        }
    }

    pub fn required(&self, name: &str) -> Result<String, String> {
        self.value(name)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| format!("{} is required", label_from_field_name(name)))
    }

    fn value(&self, name: &str) -> Option<&'a str> {
        self.values
            .and_then(|values| values.get(name))
            .map(String::as_str)
    }
}

impl DevtoolInputField for String {
    fn devtool_schema(name: &'static str) -> DevtoolFieldSchema {
        DevtoolFieldSchema::text(name, true)
    }

    fn devtool_build(name: &'static str, inputs: &DevtoolInputValues<'_>) -> Result<Self, String> {
        inputs.required(name)
    }
}

impl DevtoolInputField for bool {
    fn devtool_schema(name: &'static str) -> DevtoolFieldSchema {
        DevtoolFieldSchema::text(name, true)
    }

    fn devtool_build(name: &'static str, inputs: &DevtoolInputValues<'_>) -> Result<Self, String> {
        let value = inputs.required(name)?;
        match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "y" | "on" => Ok(true),
            "0" | "false" | "no" | "n" | "off" => Ok(false),
            _ => Err(format!("{} must be a boolean", label_from_field_name(name))),
        }
    }
}

macro_rules! devtool_numeric_input_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl DevtoolInputField for $ty {
                fn devtool_schema(name: &'static str) -> DevtoolFieldSchema {
                    DevtoolFieldSchema::text(name, true)
                }

                fn devtool_build(
                    name: &'static str,
                    inputs: &DevtoolInputValues<'_>,
                ) -> Result<Self, String> {
                    let value = inputs.required(name)?;
                    value.trim().parse::<$ty>().map_err(|_| {
                        format!("{} must be a valid number", label_from_field_name(name))
                    })
                }
            }
        )*
    };
}

devtool_numeric_input_field!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
