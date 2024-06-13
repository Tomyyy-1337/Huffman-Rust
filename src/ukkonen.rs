#[derive(Clone, Copy)]
enum Position {
    Last,
    Indx(usize),
}

#[derive(Clone)]
pub struct Node {
    pub children: Vec<(Node, Edge)>,
} 

#[derive(Clone, Copy)]
pub struct Edge {
    pub start: usize,
    pub end: Position,
}

impl Node {
    fn construct(input: &[u8]) {
        // TODO: Implement
    }


}