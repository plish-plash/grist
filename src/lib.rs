use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

type Location = &'static std::panic::Location<'static>;

#[macro_export]
macro_rules! here {
    () => {
        $crate::SourceLocation {
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}

pub struct Value<T: ?Sized> {
    last_used: Mutex<Option<Location>>,
    inner: RwLock<T>,
}

impl<T> Value<T> {
    pub const fn new(t: T) -> Self {
        Value {
            last_used: Mutex::new(None),
            inner: RwLock::new(t),
        }
    }
}
impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Value::new(Default::default())
    }
}

pub struct Res<T: ?Sized>(Arc<T>);

pub struct Obj<T: ?Sized>(Arc<Value<T>>);
pub struct WeakObj<T: ?Sized>(Weak<Value<T>>);

impl<T> Obj<T> {
    pub fn new(value: T) -> Self {
        Obj(Arc::new(Value::new(value)))
    }
}
impl<T: ?Sized> Obj<T> {
    pub fn from_rc(rc: Arc<Value<T>>) -> Self {
        Obj(rc)
    }
    pub fn rc(&self) -> &Arc<Value<T>> {
        &self.0
    }
    pub fn rc_weak(&self) -> Weak<Value<T>> {
        Arc::downgrade(&self.0)
    }
    pub fn downgrade(&self) -> WeakObj<T> {
        WeakObj::new(self.rc_weak())
    }

    #[track_caller]
    pub fn get(&self) -> RwLockReadGuard<T> {
        if let Ok(read_guard) = self.0.inner.try_read() {
            *self.0.last_used.lock().unwrap() = Some(std::panic::Location::caller());
            read_guard
        } else {
            if let Some(previous_location) = self.0.last_used.lock().unwrap().as_ref() {
                panic!(
                    "Obj<{}> already borrowed at {}",
                    std::any::type_name::<T>(),
                    previous_location
                );
            } else {
                panic!("Obj<{}> unknown error", std::any::type_name::<T>());
            }
        }
    }
    #[track_caller]
    pub fn get_mut(&self) -> RwLockWriteGuard<T> {
        if let Ok(write_guard) = self.0.inner.try_write() {
            *self.0.last_used.lock().unwrap() = Some(std::panic::Location::caller());
            write_guard
        } else {
            if let Some(previous_location) = self.0.last_used.lock().unwrap().as_ref() {
                panic!(
                    "Obj<{}> already borrowed at {}",
                    std::any::type_name::<T>(),
                    previous_location
                );
            } else {
                panic!("Obj<{}> unknown error", std::any::type_name::<T>());
            }
        }
    }
}
impl<T: ?Sized> Clone for Obj<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: ?Sized> std::hash::Hash for Obj<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state)
    }
}
impl<T: ?Sized> PartialEq for Obj<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: ?Sized> Eq for Obj<T> {}

impl<T: ?Sized> WeakObj<T> {
    pub fn new(weak: Weak<Value<T>>) -> Self {
        WeakObj(weak)
    }
    pub fn exists(&self) -> bool {
        Weak::strong_count(&self.0) > 0
    }
    pub fn try_upgrade(&self) -> Option<Obj<T>> {
        Weak::upgrade(&self.0).map(Obj::from_rc)
    }
    pub fn upgrade(&self) -> Obj<T> {
        if let Some(obj) = self.try_upgrade() {
            obj
        } else {
            panic!(
                "WeakObj<{}> object no longer exists",
                std::any::type_name::<T>()
            );
        }
    }
    pub fn rc_weak(&self) -> &Weak<Value<T>> {
        &self.0
    }
}
impl<T: ?Sized> Clone for WeakObj<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: ?Sized> std::hash::Hash for WeakObj<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Weak::as_ptr(&self.0).hash(state)
    }
}
impl<T: ?Sized> PartialEq for WeakObj<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl<T: ?Sized> Eq for WeakObj<T> {}

#[macro_export]
macro_rules! obj_upcast {
    ($obj:expr) => {
        $crate::WeakObj::new($obj.rc_weak() as _)
    };
}

pub struct Event<T> {
    listeners: Vec<Box<dyn FnMut(&T)>>,
}

impl<T> Event<T> {
    pub fn new() -> Self {
        Event {
            listeners: Vec::new(),
        }
    }
    pub fn add_listener<F>(&mut self, f: F)
    where
        F: FnMut(&T) + 'static,
    {
        self.listeners.push(Box::new(f));
    }
    pub fn emit(&mut self, param: &T) {
        for listener in self.listeners.iter_mut() {
            listener(param);
        }
    }
}

#[macro_export]
macro_rules! impl_add_event_listener {
    ($type:ty, $member:ident, $param:ty, $fn_name:ident) => {
        impl $type {
            pub fn $fn_name<F>(&mut self, f: F)
            where
                F: FnMut(&$param) + 'static,
            {
                self.$member.add_listener(f);
            }
        }
    };
}
