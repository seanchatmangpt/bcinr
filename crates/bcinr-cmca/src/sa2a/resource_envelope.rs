#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub struct ResourceEnvelope { pub cpu:u64,pub memory:u64,pub io:u64 }
#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub struct Allocation { pub cpu:u64,pub memory:u64,pub io:u64 }
impl ResourceEnvelope { pub const fn admits(&self,a:&Allocation)->bool { a.cpu<=self.cpu && a.memory<=self.memory && a.io<=self.io } pub const fn child(&self,a:&Allocation)->Option<Self>{ if self.admits(a){Some(Self{cpu:self.cpu-a.cpu,memory:self.memory-a.memory,io:self.io-a.io})}else{None} } }
