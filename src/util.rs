use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::closure::Closure;
use web_sys::{Element, Event, Node, NodeList};

pub trait NodeListExt {
    fn iter(&self) -> NodeListIterator;
}

impl NodeListExt for NodeList {
    fn iter(&self) -> NodeListIterator {
        NodeListIterator::new(self)
    }
}

pub struct NodeListIterator<'a> {
    node_list: &'a NodeList,
    index: u32,
    length: u32,
}

impl<'a> NodeListIterator<'a> {
    fn new(node_list: &'a NodeList) -> Self {
        NodeListIterator {
            node_list,
            index: 0,
            length: node_list.length(),
        }
    }
}

impl<'a> std::iter::Iterator for NodeListIterator<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let index = self.index;
            self.index += 1;
            self.node_list.get(index)
        } else {
            None
        }
    }
}

pub trait ElementExt {
    fn append(&self, ty: &str) -> Result<Element, JsValue>;
    fn attr(self, name: &str, value: &str) -> Result<Element, JsValue>;
    fn on<F: 'static + FnMut(Event)>(&self, ty: &str, callback: Option<F>) -> Result<&Element, JsValue>;
}

impl ElementExt for Element {
    fn append(&self, ty: &str) -> Result<Element, JsValue> {
        let document = self.owner_document().ok_or(JsValue::NULL)?;
        let child = document.create_element(ty)?;
        self.append_child(&child)?;
        Ok(child)
    }

    fn attr(self, name: &str, value: &str) -> Result<Element, JsValue> {
        self.set_attribute(name, value).map(|_| self)
    }

    fn on<F: 'static + FnMut(Event)>(&self, ty: &str, callback: Option<F>) -> Result<&Element, JsValue> {
        match callback {
            Some(callback) => {
                self.add_event_listener_with_callback(
                    ty,
                    Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>)
                        .as_ref()
                        .unchecked_ref(),
                ).map(|_| self)
            }
            None => {
                unimplemented!()
            }
        }
    }
}

pub trait FloatExt {
    fn round_decimals(self, decimals: u32) -> Self;
}

impl FloatExt for f32 {
    fn round_decimals(self, decimals: u32) -> Self {
        let other = 10_u32.pow(decimals) as Self;
        (self * other).round() / other
    }
}

impl FloatExt for f64 {
    fn round_decimals(self, decimals: u32) -> Self {
        let other = 10_u32.pow(decimals) as Self;
        (self * other).round() / other
    }
}
