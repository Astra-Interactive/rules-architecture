use crate::check::api::Rule;
use crate::runtime::rule::{
    BlockOnInTest, BlockOnOutsideBoundary, BlockingCallInAsync, BlockingLockInSync, BridgeCleanup,
    BroadcastLag, CallbackBridgeShape, CatchUnwind, CriticalSectionWrite, DetachedTask,
    DurationOverloadCall, ErrorLogging, IgnoredTest, ImpureUpdateClosure, JoinSetBehindLock,
    LockChoice, ModelRuleTest, ResultConsumption, SelectCancelSafety, TestTimePaused,
    ThreadPrimitive, TimePrimitive, WatchTimeout,
};
use std::sync::Arc;

pub(crate) struct RuntimeRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl RuntimeRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(BlockOnOutsideBoundary),
            Arc::new(BlockOnInTest),
            Arc::new(IgnoredTest),
            Arc::new(DetachedTask),
            Arc::new(BridgeCleanup),
            Arc::new(CallbackBridgeShape),
            Arc::new(BlockingCallInAsync),
            Arc::new(ResultConsumption),
            Arc::new(BlockingLockInSync),
            Arc::new(BroadcastLag),
            Arc::new(DurationOverloadCall),
            Arc::new(ImpureUpdateClosure),
            Arc::new(WatchTimeout),
            Arc::new(LockChoice),
            Arc::new(JoinSetBehindLock),
            Arc::new(TestTimePaused),
            Arc::new(ModelRuleTest),
            Arc::new(ThreadPrimitive),
            Arc::new(CatchUnwind),
            Arc::new(TimePrimitive),
            Arc::new(ErrorLogging),
            Arc::new(SelectCancelSafety),
            Arc::new(CriticalSectionWrite),
        ];
        let active_rule_ids = vec![
            "BlockOnOutsideBoundary",
            "BlockOnInTest",
            "IgnoredTest",
            "DetachedTask",
            "BridgeCleanup",
            "CallbackBridgeShape",
            "BlockingCallInAsync",
            "ResultConsumption",
            "BlockingLockInSync",
            "BroadcastLag",
            "DurationOverloadCall",
            "ImpureUpdateClosure",
            "WatchTimeout",
            "LockChoice",
            "JoinSetBehindLock",
            "TestTimePaused",
            "ModelRuleTest",
            "ThreadPrimitive",
            "CatchUnwind",
            "TimePrimitive",
            "ErrorLogging",
            "SelectCancelSafety",
            "CriticalSectionWrite",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
