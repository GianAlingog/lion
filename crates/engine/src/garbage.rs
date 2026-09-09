pub struct GarbageBatch {
    pub lines: u8,
    pub hole: u8,
    pub delay: u16,
}

pub struct GarbageQueue {
    // Vec over VecDeque since this should remain small in size
    _pending: Vec<GarbageBatch>,
}

impl GarbageQueue {
    pub fn incoming(&self) -> u32 {
        todo!("Return number of incoming lines in the queue")
    }

    pub fn push(&mut self, _batch: GarbageBatch) {
        todo!("Push the garbage to the vec; handle other initializations in the future")
    }

    pub fn cancel(&mut self, _amount: u32) -> u32 {
        todo!("Should return number of attach left, use in while loop")
    }

    pub fn tick(&mut self) -> Vec<GarbageBatch> {
        todo!("Return the batches that are to be pushed")
    }
}
