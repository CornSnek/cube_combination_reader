pub struct FloatStep{init:f64, end:f64, c_now:isize, bc_now:isize, end_c:isize, cmp_c_func:fn(&isize,&isize)->bool}
impl FloatStep{
    ///Iterates from init to end in n number of steps from [init,end)
    pub fn new_noend(init:f64, end:f64, n:isize)->Self{
        Self{init, end, c_now:0, bc_now:n-1, end_c:n, cmp_c_func:isize::lt}
    }
    ///Iterates from init to end in n+1 number of steps from [init,end]
    pub fn new_wend(init:f64,end:f64,n:isize)->Self{
        Self{init, end, c_now:0, bc_now:n, end_c:n, cmp_c_func:isize::le}
    }
}
impl Iterator for FloatStep{
    type Item=f64;
    fn next(&mut self)->Option<Self::Item> {
        if (self.cmp_c_func)(&self.c_now,&self.end_c)&&self.c_now<=self.bc_now{
            let res=Some(self.init+(self.end-self.init)/self.end_c as f64*self.c_now as f64);
            self.c_now+=1;
            res
        }else{
            None
        }
    }
}
impl DoubleEndedIterator for FloatStep{
    fn next_back(&mut self)->Option<Self::Item>{
        if (self.cmp_c_func)(&0,&self.bc_now)&&self.c_now<=self.bc_now{
            let res=Some(self.init+(self.end-self.init)/self.end_c as f64*self.bc_now as f64);
            self.bc_now-=1;
            res
        }else{
            None
        }
    }
}
impl ExactSizeIterator for FloatStep{
    fn len(&self) -> usize {
        (self.bc_now-self.c_now+1) as usize
    }
}
pub struct ScaleOffsetSize{
    scale_x:f32,
    scale_y:f32,
    offset_x:i32,
    offset_y:i32,
}
impl ScaleOffsetSize{
    pub fn new(scale_x:f32,scale_y:f32,offset_x:i32,offset_y:i32)->Self{
        Self{scale_x,scale_y,offset_x,offset_y}
    }
    #[inline] pub fn p(&self,parent_p:(i32,i32),parent_s:(i32,i32))->(i32,i32){ (self.x(parent_p,parent_s),self.y(parent_p,parent_s)) }
    #[inline] pub fn x(&self,parent_p:(i32,i32),parent_s:(i32,i32))->i32{ ((parent_p.0+parent_s.0) as f32 * self.scale_x) as i32 + self.offset_x }
    #[inline] pub fn y(&self,parent_p:(i32,i32),parent_s:(i32,i32))->i32{ ((parent_p.1+parent_s.1) as f32 * self.scale_y) as i32 + self.offset_y }
}