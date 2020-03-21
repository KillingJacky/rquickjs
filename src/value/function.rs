use crate::{value::rf::JsObjectRef, FromJs, Object, Result, ToJsMulti, Value};
use rquickjs_sys as qjs;
use std::mem;

/// Rust representation of a javascript function.
#[derive(Debug, Clone, PartialEq)]
pub struct Function<'js>(pub(crate) JsObjectRef<'js>);

impl<'js> Function<'js> {
    /// Call a function with given arguments with this as the global object.
    pub fn call<A, R>(&self, args: A) -> Result<R>
    where
        A: ToJsMulti<'js>,
        R: FromJs<'js>,
    {
        let args = args.to_js_multi(self.0.ctx)?;
        let len = args.len();
        let res = unsafe {
            // Dont drop args value
            let mut args: Vec<_> = args.iter().map(|x| x.as_js_value()).collect();
            let val = qjs::JS_Call(
                self.0.ctx.ctx,
                self.0.as_js_value(),
                self.0.ctx.globals().0.as_js_value(),
                len as i32,
                args.as_mut_ptr(),
            );
            R::from_js(self.0.ctx, Value::from_js_value(self.0.ctx, val)?)
        };
        // Make sure the lifetime of args remains valid during the
        // entire duration of the call.
        mem::drop(args);
        res
    }

    pub fn to_object(self) -> Object<'js> {
        Object(self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    fn base_call() {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let f: Function = ctx.eval("(a) => a + 4").unwrap();
            let res = f.call(3).unwrap();
            println!("{:?}", res);
            assert_eq!(i32::from_js(ctx, res).unwrap(), 7);
            let f: Function = ctx.eval("(a,b) => a * b + 4").unwrap();
            let res = f.call((3, 4)).unwrap();
            println!("{:?}", res);
            assert_eq!(i32::from_js(ctx, res).unwrap(), 16);
        })
    }
}
