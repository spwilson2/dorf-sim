////
////use std::collections::HashMap;
////use std::any::TypeId;
//use std::sync::Arc;
////use std::collections::vec_deque::VecDeque;
////use std::collections::vec_deque::IterMut;
////
////pub trait System {
////    fn run();
////}
////
////pub trait Entity {
////    fn new() -> Self;
////}
////
////pub trait DefaultEntity {}
////impl<T> Entity for T where T: Default + DefaultEntity {
////    fn new() -> Self {
////        Default::default()
////    }
////}
////
//
//use bevy_ecs::prelude::*;
//
//#[derive(Default)]
//pub struct App {
//    schedule: Box<Schedule>,
//    world: Box<World>,
//}
//
//#[derive(Default)]
//pub struct AppState {}
//
//impl App {
//    pub fn new() -> Self {
//        Self {
//            ..Default::default()
//        }
//    }
//    pub fn add_plugin<T: Plugin>(&mut self) -> &mut Self {
//        T::build(self);
//        self
//    }
//
//    pub fn add_system<M>(
//        &mut self,
//        system: impl IntoSystemConfig<M, bevy_ecs::schedule::SystemConfig>,
//    ) -> &mut Schedule {
//        self.schedule.add_system(system)
//    }
//
//    pub fn run(&mut self) {
//        self.schedule.run(&mut self.world)
//    }
//}
//
//pub trait Plugin {
//    fn build(app: &mut App);
//}
////
////
////// Container for ECS data.
////pub struct World {
////    entities: HashMap<TypeId, Box<dyn std::any::Any>>,
////}
////
////impl World {
////    pub fn new() -> Self {
////        Self{
////            entities: Default::default()
////        }
////    }
////    fn container<T: 'static>(&mut self) -> Option<&mut VecDeque<T>> {
////        self.entities.get_mut(&TypeId::of::<T>()).and_then(|v_c| unsafe {Some(v_c.downcast_mut::<VecDeque<T>>()).unwrap_unchecked()})
////    }
////
////    pub fn entities<T: 'static>(&mut self) -> Option<IterMut<T>> {
////        self.container().and_then(|f| Some(f.iter_mut()))
////    }
////
////    pub fn add<T: 'static>(&mut self, item: T) {
////        let key = TypeId::of::<T>();
////        match self.entities.get_mut(&key) {
////            Some(d) => {unsafe {d.downcast_mut::<VecDeque<T>>().unwrap_unchecked().push_back(item);}},
////            None => {
////                let mut d = VecDeque::<T>::new();
////                d.push_back(item);
////                self.entities.insert(key, Box::new(d));
////            },
////        };
////    }
////
////    // TODO: I might want to add a light wrapper type around Entities in order to make tracking of them simpler.. (e.g. provide a UID/index into their container.
////    fn remove() {}
////
////}
////
//
////#[test]
////fn test_simple_storage() {
////    let mut w = World::new();
////    w.add(2);
////    w.add("world");
////    assert_eq!(w.entities::<i32>().unwrap().map(|i| *i).collect::<Vec<i32>>(), [2]);
////}
//