#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::{time::Instant, collections::HashMap, ops};

use crate::buffer::Mode;




#[derive(Clone)]
pub enum QuickAction {
    QuickNormal, 
    MakeBlock, 
}
#[derive(Clone)]
pub enum Operation {
    ToInsert, 
    ToNormal, 
    EnterCmd, 

    Delete, 
    NewLine, 
}



#[derive(Clone)]
pub struct OperationBuffer {
    pub imacros :   HashMap<String, QuickAction>, 
    // ops     :   HashMap<String, Operation>, 
    pub currMac :   Vec<char>, 
    pub currOp  :   Vec<Operation>, 
    pub lastOp  :   Vec<Operation>, 
    pub lastInp :   Instant, }

// CONSTRUCTORS & PRIV FUNCS \\
impl OperationBuffer {
    pub fn new() -> Self {
        let mut imacros = HashMap::from([
            (String::from("jk"), QuickAction::QuickNormal), 
            (String::from("{\n"), QuickAction::MakeBlock),
        ]);

        // let mut ops = HashMap::from([
        // ]);

        Self {
            imacros, 
            // ops, 
            currMac :   Vec::new(), 
            currOp  :   Vec::new(), 
            lastOp  :   Vec::new(), 
            lastInp :   Instant::now(), 
        } }
}

// PUB FUNCS \\ 
impl OperationBuffer {
    // pub fn checkOperation(self: &mut Self, operation: Operation) {
        
    // }

    pub fn checkMacro(self: &mut Self, c: char) -> bool {

        true
    }
}
