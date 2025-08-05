#![expect(clippy::box_collection)]
#![expect(unused)]

use std::{
    borrow::Cow,
    rc::{self, Rc},
    sync::{self, Arc},
};

use facet::Facet;

/// This is a comment.
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum BoxyColors {
    Red,
    Blue,
    Green(Box<String>),
}

/// This is a comment.
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub struct ArcyColors {
    pub red: rc::Weak<u8>,
    pub blue: sync::Weak<String>,
    pub green: Arc<Vec<String>>,
}

// /// This is a comment.
// #[derive(Facet)]
// #[facet(tag = "type", content = "content")]
// pub struct MutexyColors {
//     pub blue: Mutex<Vec<String>>,
//     pub green: Mutex<String>,
// }

/// This is a comment.
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub struct RcyColors {
    pub red: rc::Weak<String>,
    pub blue: Rc<Vec<String>>,
    pub green: Rc<String>,
}

// /// This is a comment.
// #[derive(Facet)]
// #[facet(tag = "type", content = "content")]
// pub struct CellyColors {
//     pub red: Cell<String>,
//     pub blue: RefCell<Vec<String>>,
// }

// /// This is a comment.
// #[derive(Facet)]
// #[facet(tag = "type", content = "content")]
// pub struct LockyColors {
//     pub red: RwLock<String>,
// }

/// This is a comment.
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub struct CowyColors<'a> {
    pub lifetime: Cow<'a, str>,
}
