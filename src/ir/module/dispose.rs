use crate::ir::IPoolAllocated;
use mtb_entity::{EntityList, IEntityListNode};

pub(crate) fn dispose_entity_list<T>(list: &EntityList<T>, pool: &T::MinRelatedPoolT)
where
    T: IPoolAllocated + IEntityListNode,
{
    let alloc = T::get_alloc(pool.as_ref());
    while let Ok(id) = list.pop_front(alloc) {
        let id = T::make_module_id(id);
        T::dispose_id(id, pool);
    }
    let head = T::make_module_id(list.head);
    let tail = T::make_module_id(list.tail);
    T::dispose_id(head, pool);
    T::dispose_id(tail, pool);
}
