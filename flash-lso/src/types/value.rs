use super::{ClassDefinition, Element, ObjectId, Reference};
use std::rc::Rc;

//TODO: should amf3 assoc arrays be their own type with a dense and assoc section
/// A single or compound value
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represent the type number (amf0) and double (amf3)
    Number(f64),

    /// Represents the type boolean (amf0) and both the true/false type (amf3)
    Bool(bool),

    /// Represent both the string (amf0/3) and long string type (amf0)
    String(String),

    /// Represents the object type in both amf0 and amf3, class definition are only available with amf3
    Object(ObjectId, Vec<Element>, Option<ClassDefinition>),

    /// Represent the null type
    Null,

    /// Represent the undefined type
    Undefined,

    /// Represent ECMA-Arrays (amf0) and associative arrays (amf3, even if they contain a dense part)
    /// Final value represents the length of the array in amf0, this can differ from the actual number of elements
    ECMAArray(ObjectId, Vec<Rc<Value>>, Vec<Element>, u32),

    /// Represent a strict array (amf0) or a dense array (amf3)
    StrictArray(ObjectId, Vec<Rc<Value>>),

    /// Represent a timezone in the format (seconds since epoch, timezone or UTC if missing (amf3) )
    Date(f64, Option<u16>),

    /// Represent the unsupported type
    Unsupported,

    /// Represent the XML type, (value, is_string)
    XML(String, bool),

    #[cfg(feature = "amf3")]
    /// Represent an amf3 element embedded in an AMF0 file
    AMF3(Rc<Value>),

    // AMF3
    /// Represent the integer type (u29) (amf3)
    Integer(i32),

    /// Represent the bytearray type (amf3)
    ByteArray(Vec<u8>),

    /// Represent the int vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorInt(Vec<i32>, bool),

    /// Represent the unsigned int vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorUInt(Vec<u32>, bool),

    /// Represent the double vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorDouble(Vec<f64>, bool),

    /// Represent the object vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorObject(ObjectId, Vec<Rc<Value>>, String, bool),

    /// Represent the dictionary type (amf3)
    /// Format is ((key, value), has_weak_keys)
    Dictionary(ObjectId, Vec<(Rc<Value>, Rc<Value>)>, bool),

    /// Represent a external object, such as from flex
    /// (custom_elements, regular elements, class def)
    Custom(Vec<Element>, Vec<Element>, Option<ClassDefinition>),

    /// Represent an existing value, stored by reference, the value here should be considered opaque
    Reference(Reference),

    /// A reference to a previously parsed element
    ///
    /// While traversing `Value`s you should maintain a mapping of `ObjectId` to your internal
    /// representation of a value and consider this a reference to the exact same value.
    ///
    /// As `Value` graphs can contain cycles which are best handled by garbage collected structures
    /// we leave the handling of this to the user, sorry
    Amf3ObjectReference(ObjectId),
}

impl FromIterator<Value> for Vec<Rc<Value>> {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        iter.into_iter().map(Rc::new).collect()
    }
}
