use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

use crate::attribute;
use crate::datamodel::DataModel;
use crate::object::{self, Enumeration};
use crate::validation::Validator;

use super::frontmatter::parse_frontmatter;

/// Parses a Markdown file at the given path and returns a `DataModel`.
///
/// # Arguments
///
/// * `path` - A reference to the path of the Markdown file.
///
/// # Returns
///
/// A `Result` containing a `DataModel` on success or an error on failure.
pub fn parse_markdown(path: &Path) -> Result<DataModel, Box<dyn Error>> {
    if !path.exists() {
        return Err("File does not exist".into());
    }

    let content = fs::read_to_string(path).expect("Could not read file");

    // Remove all html tags
    let re = Regex::new(r"<[^>]*>").unwrap();
    let content = re.replace_all(&content, "").to_string();

    // Parse the frontmatter
    let config = parse_frontmatter(content.as_str());

    // Parse the markdown content
    let parser = Parser::new(&content);
    let mut iterator = parser.into_iter();

    let mut objects = Vec::new();
    let mut enums = Vec::new();

    let mut model = DataModel::new(None, config);

    // Extract objects from the markdown file
    while let Some(event) = iterator.next() {
        process_object_event(&mut iterator, &mut objects, event, &mut model);
    }

    // Reset the iterator
    let parser = Parser::new(&content);
    let mut iterator = parser.into_iter();

    while let Some(event) = iterator.next() {
        process_enum_event(&mut iterator, &mut enums, event);
    }

    model.enums = enums.into_iter().filter(|e| e.has_values()).collect();
    model.objects = objects.into_iter().filter(|o| o.has_attributes()).collect();

    // Validate the model
    let mut validator = Validator::new();
    validator.validate(&model)?;

    Ok(model)
}

/// Processes a single Markdown event for object extraction.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
/// * `objects` - A mutable reference to the vector of objects.
/// * `event` - The current Markdown event.
/// * `model` - A mutable reference to the data model.
fn process_object_event(
    iterator: &mut Parser,
    objects: &mut Vec<object::Object>,
    event: Event,
    model: &mut DataModel,
) {
    match event {
        Event::Start(Tag::Heading(1)) => {
            model.name = Some(extract_name(iterator));
        }
        Event::Start(Tag::Heading(3)) => {
            let object = process_object_heading(iterator);
            objects.push(object);
        }
        Event::Start(Tag::List(None)) => {
            let last_object = objects.last_mut().unwrap();
            if !last_object.has_attributes() {
                iterator.next();
                let (required, attr_name) = extract_attr_name_required(iterator);
                let attribute = attribute::Attribute::new(attr_name, required);
                objects.last_mut().unwrap().add_attribute(attribute);
            } else {
                let attr_strings = extract_attribute_options(iterator);
                for attr_string in attr_strings {
                    distribute_attribute_options(objects, attr_string);
                }
            }
        }
        Event::Start(Tag::Item) => {
            let (required, attr_string) = extract_attr_name_required(iterator);
            let attribute = attribute::Attribute::new(attr_string, required);
            objects.last_mut().unwrap().add_attribute(attribute);
        }
        _ => {}
    }
}

/// Processes the heading of an object.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// An `Object` created from the heading.
fn process_object_heading(iterator: &mut Parser) -> object::Object {
    let heading = extract_name(iterator);
    let term = extract_object_term(&heading);
    let name = heading.split_whitespace().next().unwrap().to_string();

    object::Object::new(name, term)
}

/// Extracts the name from the next text event in the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A string containing the extracted name.
fn extract_name(iterator: &mut Parser) -> String {
    if let Some(Event::Text(text)) = iterator.next() {
        return text.to_string();
    }

    panic!("Could not extract name: Got {:?}", iterator.next().unwrap());
}

/// Extracts the attribute name and its required status from the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A tuple containing a boolean indicating if the attribute is required and the attribute name.
fn extract_attr_name_required(iterator: &mut Parser) -> (bool, String) {
    if let Some(Event::Text(text)) = iterator.next() {
        return (false, text.to_string());
    }

    // Try for two text events
    for _ in 0..2 {
        if let Some(Event::Text(text)) = iterator.next() {
            return (true, text.to_string());
        }
    }

    panic!("Could not extract name. Plesae check the markdown file.");
}

/// Extracts the term from an object heading.
///
/// # Arguments
///
/// * `heading` - A string slice containing the heading.
///
/// # Returns
///
/// An optional string containing the extracted term.
fn extract_object_term(heading: &str) -> Option<String> {
    let re = Regex::new(r"\(([^)]+)\)").unwrap();

    re.captures(heading)
        .map(|cap| cap.get(1).map_or("", |m| m.as_str()).to_string())
}

/// Extracts attribute options from the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A vector of strings containing the extracted attribute options.
fn extract_attribute_options(iterator: &mut Parser) -> Vec<String> {
    let mut options = Vec::new();
    while let Some(next) = iterator.next() {
        match next {
            Event::Start(Tag::Item) => {
                let name = extract_name(iterator);
                options.push(name);
            }
            Event::End(Tag::List(None)) => {
                break;
            }
            Event::Text(text) if text.to_string() == "[" => {
                let last_option = options.last_mut().unwrap();
                *last_option = format!("{}[]", last_option);
            }
            _ => {}
        }
    }

    options
}

/// Adds an option to the last attribute of the last object in the list.
///
/// # Arguments
///
/// * `objects` - A mutable reference to the list of objects.
/// * `key` - The key of the attribute option.
/// * `value` - The value of the attribute option.
fn add_option_to_last_attribute(objects: &mut [object::Object], key: String, value: String) {
    let last_attr = objects.last_mut().unwrap().get_last_attribute();
    let option = attribute::AttrOption::new(key, value);
    last_attr.add_option(option);
}

/// Distributes attribute options among the objects.
///
/// # Arguments
///
/// * `objects` - A mutable reference to the list of objects.
/// * `attr_string` - A string containing the attribute or option.
///
/// # Returns
///
/// An optional unit type.
fn distribute_attribute_options(objects: &mut [object::Object], attr_string: String) -> Option<()> {
    if attr_string.contains(':') {
        let (key, value) = process_option(&attr_string);
        add_option_to_last_attribute(objects, key, value);
        return None;
    }

    objects
        .last_mut()
        .unwrap()
        .create_new_attribute(attr_string, false);

    None
}

/// Processes an attribute option string.
///
/// # Arguments
///
/// * `option` - A string containing the attribute option.
///
/// # Returns
///
/// A tuple containing the key and value of the attribute option.
fn process_option(option: &String) -> (String, String) {
    let parts: Vec<&str> = option.split(':').collect();

    assert!(
        parts.len() > 1,
        "Attribute {} does not have a valid option",
        option
    );

    let key = parts[0].trim();
    let value = parts[1..].join(":");

    (key.to_string(), value.trim().to_string())
}

/// Processes a single Markdown event for enumeration extraction.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
/// * `enums` - A mutable reference to the vector of enumerations.
/// * `event` - The current Markdown event.
pub fn process_enum_event(iterator: &mut Parser, enums: &mut Vec<Enumeration>, event: Event) {
    match event {
        Event::Start(Tag::Heading(3)) => {
            let enum_name = extract_name(iterator);
            let enum_obj = Enumeration {
                name: enum_name,
                mappings: BTreeMap::new(),
            };
            enums.push(enum_obj);
        }
        Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(_))) => {
            let event = iterator.next().unwrap();
            if let Event::Text(text) = event {
                let mappings = text.to_string();
                let enum_obj = enums.last_mut().unwrap();
                process_enum_mappings(enum_obj, mappings);
            }
        }
        _ => {}
    }
}

/// Processes enumeration mappings from a code block.
///
/// # Arguments
///
/// * `enum_obj` - A mutable reference to the enumeration object.
/// * `mappings` - A string containing the mappings.
fn process_enum_mappings(enum_obj: &mut Enumeration, mappings: String) {
    let lines = mappings.split('\n');
    for line in lines {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            // Skip empty lines or lines that do not contain a mapping
            continue;
        }

        // Extract key and value, insert into enum object
        let key = parts[0].trim().replace('"', "");
        let value = parts[1].trim().replace('"', "");
        enum_obj.mappings.insert(key.to_string(), value.to_string());
    }
}
