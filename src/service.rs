use crate::actor::ActorManager;
use crate::error::Result;
use crate::{Actor, Addr};
use fnv::FnvHasher;
use futures::lock::Mutex;
use once_cell::sync::OnceCell;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
#[async_trait::async_trait]
pub trait Service: Actor + Default {
    async fn from_registry() -> Result<Addr<Self>> {
        static REGISTRY: OnceCell<
            Mutex<HashMap<TypeId, Box<dyn Any + Send>, BuildHasherDefault<FnvHasher>>>,
        > = OnceCell::new();
        let registry = REGISTRY.get_or_init(Default::default);
        let mut registry = registry.lock().await;

        match registry.get_mut(&TypeId::of::<Self>()) {
            Some(addr) => Ok(addr.downcast_ref::<Addr<Self>>().unwrap().clone()),
            None => {
                let actor_manager = ActorManager::new();

                registry.insert(TypeId::of::<Self>(), Box::new(actor_manager.address()));
                drop(registry);

                actor_manager.start_actor(Self::default()).await
            }
        }
    }
}

thread_local! {
    static LOCAL_REGISTRY: RefCell<HashMap<TypeId, Box<dyn Any + Send>, BuildHasherDefault<FnvHasher>>> = Default::default();
}
/// Trait define a local service.
///
/// The service is a thread local actor.
/// You can use `Actor::from_registry` to get the address `Addr<A>` of the service.
#[async_trait::async_trait]
pub trait LocalService: Actor + Default {
    async fn from_registry() -> Result<Addr<Self>> {
        let res = LOCAL_REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .get_mut(&TypeId::of::<Self>())
                .map(|addr| addr.downcast_ref::<Addr<Self>>().unwrap().clone())
        });
        match res {
            Some(addr) => Ok(addr),
            None => {
                let addr = ActorManager::new().start_actor(Self::default()).await?;
                LOCAL_REGISTRY.with(|registry| {
                    registry
                        .borrow_mut()
                        .insert(TypeId::of::<Self>(), Box::new(addr.clone()));
                });
                Ok(addr)
            }
        }
    }
}
