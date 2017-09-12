//Wrap xml-rs for ease of use and better readability
//WARNING: It loads your entire XML file into memory

use std::ops::Index;
use std::borrow::Borrow;
use std::string::String;
use std::collections::HashMap;
use xml::reader::{EventReader, XmlEvent};

pub struct XmlNode{
    //FIXME: Right now, as is written, using the Parent var will cause Heap Corruption
    // parent: Option<Box<XmlNode>>,
    pub value: String,
    children: HashMap<String,Vec<Box<XmlNode>>>,
}

impl XmlNode{
    fn new() -> XmlNode{
        return XmlNode{
            // parent: None,
            value: String::new(),
            children: HashMap::new(),
        }
    }

    fn boxed() -> Box<XmlNode>{
        return Box::new(XmlNode::new());
    }

    fn add_new_child(&mut self, key: String, val: String) -> &XmlNode{
        // unsafe{
        let mut new_node = Box::new(XmlNode { /*parent: Some(Box::from_raw(self)),*/ value: val, children: HashMap::new()});
        let mut list = self.children.entry(key).or_insert(Vec::new());
        list.push(new_node);
        return list.last().unwrap();
        // }
    }

    fn add_child(&mut self, key: String, mut node: Box<XmlNode>) -> &XmlNode{
        // unsafe{
            // node.parent = Some(Box::from_raw(self));
        let mut list = self.children.entry(key).or_insert(Vec::new());
        list.push(node);
        return list.last().unwrap();
        // }
    }
}

impl Index<usize> for XmlNode {
    type Output = String;

    fn index(&self, idx: usize) -> &String {
        //Panic when there's more than one child
        if self.children.len() > 1{
            panic!("ERROR: Attempting to use numbered index on a node with more than one child. Were you expecting a list?");
        }

        for child in self.children.values() {
            return &(*child)[idx].value;
        }

        panic!("Index not found")
    }
}

impl<'a> Index<&'a str> for XmlNode {
    type Output = XmlNode;

    fn index(&self, idx: &'a str) -> &XmlNode {
        let entry = self.children.get(&String::from(idx)).unwrap();
        return &(*entry[0]); 
    }
}

pub struct XmlMap{
    pub root: Box<XmlNode>,
    //Debug only
    raw_data: String,
}

impl XmlMap{
    fn create(xml_parser: &mut EventReader<&[u8]>) -> Box<XmlNode>{
        let mut ret_val = XmlNode::boxed();
        let mut is_done = false;
        //TODO: Check and finish
        while !is_done {
            let e = xml_parser.next();
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    debug!("Adding {}", name.local_name);
                    let mut child = XmlMap::create(xml_parser);
                    ret_val.add_child(name.local_name, child);
                }
                Ok(XmlEvent::Characters(data)) => {
                    debug!("data {}", data);
                    ret_val.value = data;
                }
                Ok(XmlEvent::EndElement { name }) => {
                    debug!("Ending {}", name.local_name);
                    return ret_val;
                }
                Ok(XmlEvent::EndDocument { .. }) => {
                    debug!("Finished");
                    return ret_val;
                }
                Err(e) => {
                    error!("{}",e.msg());
                    is_done =true;
                }
                _ => {}
            }
        }

        ret_val
    }

    pub fn from_str(raw_data : &str) -> XmlMap{
        let mut streaming_parser = EventReader::from_str(raw_data);
        let mut root = XmlMap::create(&mut streaming_parser);
        let mut dstruct = XmlMap{ root: root, raw_data: String::from(raw_data)};
        return dstruct;
    }
}