/// An internal little, chainable, and faster version of `std::fmt::Write`
pub trait FastWrite {
    fn write(&mut self, val: char);
    fn write_str(&mut self, val: &str);

    #[inline(always)]
    fn write_int(&mut self, val: impl itoa::Integer) {
        self.write_str(itoa::Buffer::new().format(val))
    }

    #[inline(always)]
    fn write_float(&mut self, val: impl zmij::Float) {
        self.write_str(zmij::Buffer::new().format(val))
    }

    #[inline(always)]
    fn write_if(&mut self, cond: bool, val: char) {
        if cond {
            self.write(val)
        }
    }

    #[inline(always)]
    fn write_str_if(&mut self, cond: bool, val: &str) {
        if cond {
            self.write_str(val)
        }
    }

    #[inline(always)]
    fn write_int_if(&mut self, cond: bool, val: impl itoa::Integer) {
        if cond {
            self.write_int(val)
        }
    }

    #[inline(always)]
    fn write_float_if(&mut self, cond: bool, val: impl zmij::Float) {
        if cond {
            self.write_float(val)
        }
    }
}

impl FastWrite for String {
    #[inline(always)]
    fn write(&mut self, val: char) {
        self.push(val);
    }

    #[inline(always)]
    fn write_str(&mut self, val: &str) {
        self.push_str(val);
    }
}
