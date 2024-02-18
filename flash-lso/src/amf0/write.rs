/// Support for encoding AMF0
use crate::types::{Element, Reference, Value};
use crate::PADDING;
use cookie_factory::bytes::{be_f64, be_u16, be_u32, be_u8};
use cookie_factory::{SerializeFn, WriteContext};
use std::io::Write;

use crate::amf0::type_marker::TypeMarker;
use crate::amf3::write::AMF3Encoder;
use crate::nom_utils::write_string;
use cookie_factory::combinator::slice;
use cookie_factory::combinator::string;
use cookie_factory::multi::all;
use cookie_factory::sequence::tuple;
use std::ops::Deref;
use std::rc::Rc;

fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(type_: TypeMarker) -> impl SerializeFn<W> + 'a {
    be_u8(type_ as u8)
}

fn write_reference_element<'a, 'b: 'a, W: Write + 'a>(r: &Reference) -> impl SerializeFn<W> + 'a {
    tuple((write_type_marker(TypeMarker::Reference), be_u16(r.0)))
}

fn write_number_element<'a, 'b: 'a, W: Write + 'a>(s: f64) -> impl SerializeFn<W> + 'a {
    tuple((write_type_marker(TypeMarker::Number), be_f64(s)))
}

fn write_bool_element<'a, 'b: 'a, W: Write + 'a>(s: bool) -> impl SerializeFn<W> + 'a {
    tuple((write_type_marker(TypeMarker::Boolean), be_u8(u8::from(s))))
}

fn write_long_string_content<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(s.len() as u32), string(s)))
}

fn write_long_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
    tuple((
        write_type_marker(TypeMarker::LongString),
        write_long_string_content(s),
    ))
}

fn write_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
    tuple((write_type_marker(TypeMarker::String), write_string(s)))
}

fn write_object_element<'a, 'b: 'a, W: Write + 'a>(o: &'b [Element]) -> impl SerializeFn<W> + 'a {
    tuple((
        write_type_marker(TypeMarker::Object),
        all(o.iter().map(write_element)),
        be_u16(0),
        write_type_marker(TypeMarker::ObjectEnd),
    ))
}

fn write_null_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
    write_type_marker(TypeMarker::Null)
}

fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
    write_type_marker(TypeMarker::Undefined)
}

fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(
    elements: &'b [Rc<Value>],
) -> impl SerializeFn<W> + 'a {
    tuple((
        write_type_marker(TypeMarker::Array),
        be_u32(elements.len() as u32),
        all(elements.iter().map(write_value)),
    ))
}

fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
    date: f64,
    tz: Option<u16>,
) -> impl SerializeFn<W> + 'a {
    tuple((
        write_type_marker(TypeMarker::Date),
        be_f64(date),
        be_u16(tz.unwrap_or(0)),
    ))
}

fn write_unsupported_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
    write_type_marker(TypeMarker::Unsupported)
}

fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(content: &'b str) -> impl SerializeFn<W> + 'a {
    tuple((
        write_type_marker(TypeMarker::Xml),
        write_long_string_content(content),
    ))
}

fn write_typed_object_element<'a, 'b: 'a, W: Write + 'a>(
    name: &'b str,
    elements: &'b [Element],
) -> impl SerializeFn<W> + 'a {
    tuple((
        write_type_marker(TypeMarker::TypedObject),
        write_string(name),
        all(elements.iter().map(write_element)),
        be_u16(0),
        write_type_marker(TypeMarker::ObjectEnd),
    ))
}

fn write_dense_element<'a, 'b: 'a, W: Write + 'a>(
    index: usize,
    element: &'b Rc<Value>,
) -> impl SerializeFn<W> + 'a {
    move |ctx| {
        let index_str = index.to_string();
        tuple((
            be_u16(index_str.len() as u16),
            string(index_str),
            write_value(element),
        ))(ctx)
    }
}

fn write_mixed_array<'a, 'b: 'a, W: Write + 'a>(
    dense: &'b [Rc<Value>],
    elements: &'b [Element],
    length: u32,
) -> impl SerializeFn<W> + 'a {
    //TODO: what is the u16 padding
    //TODO: sometimes array length is ignored (u32) sometimes its: elements.len() as u32

    tuple((
        write_type_marker(TypeMarker::MixedArrayStart),
        be_u32(length),
        all(dense
            .iter()
            .enumerate()
            .map(|(idx, value)| write_dense_element(idx, value))),
        all(elements.iter().map(write_element)),
        be_u16(0),
        write_type_marker(TypeMarker::ObjectEnd),
    ))
}

pub(crate) fn write_value<'a, 'b: 'a, W: Write + 'a>(
    element: &'b Rc<Value>,
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match element.deref() {
        Value::Number(n) => write_number_element(*n)(out),
        Value::Bool(b) => write_bool_element(*b)(out),
        Value::String(s) => {
            if s.len() > 65535 {
                write_long_string_element(s)(out)
            } else {
                write_string_element(s)(out)
            }
        }
        Value::Object(elements, class_def) => {
            if let Some(class_def) = class_def {
                write_typed_object_element(&class_def.name, elements)(out)
            } else {
                write_object_element(elements)(out)
            }
        }
        Value::Null => write_null_element()(out),
        Value::Undefined => write_undefined_element()(out),
        Value::StrictArray(a) => write_strict_array_element(a.as_slice())(out),
        Value::Date(d, tz) => write_date_element(*d, *tz)(out),
        Value::Unsupported => write_unsupported_element()(out),
        Value::XML(x, _string) => write_xml_element(x)(out),
        Value::ECMAArray(dense, elems, elems_length) => {
            write_mixed_array(dense, elems, *elems_length)(out)
        }
        Value::AMF3(e) => tuple((
            write_type_marker(TypeMarker::AMF3),
            AMF3Encoder::default().write_value_element(e),
        ))(out),
        Value::Reference(r) => write_reference_element(r)(out),
        _ => {
            write_unsupported_element()(out) /* Not in amf0, TODO: use the amf3 embedding for every thing else */
        }
    }
}

fn write_element<'a, 'b: 'a, W: Write + 'a>(element: &'b Element) -> impl SerializeFn<W> + 'a {
    tuple((write_string(&element.name), write_value(&element.value)))
}

fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
    element: &'b Element,
) -> impl SerializeFn<W> + 'a {
    tuple((write_element(element), slice(PADDING)))
}

pub(crate) fn write_body<'a, 'b: 'a, W: Write + 'a>(
    elements: &'b [Element],
) -> impl SerializeFn<W> + 'a {
    all(elements.iter().map(write_element_and_padding))
}
