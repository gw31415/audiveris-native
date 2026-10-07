# Next dependency

Stable SIG edge identity is now separate from current live insertion order.
Original same-edge remove/re-add moves it behind remaining live supports, while
adding an already-live edge leaves order unchanged. Original null first-supported
Part is returned without fallback; changing live order changes the selected
owner. Incoming edge copy precedes outgoing copy, each in current live order.
Every removed relation callback compares actual owner counts and dot/cache /
Measure state. This is saved-state control, not new shared-image success.

Next original dependency: AugmentationRelation.added invokes the source abnormal
check immediately, then recounts the actual target chord if the relation or dot
is manual. Current ordinary automatic added support only refreshes source state;
the manual branch and per-added-edge callbacks in batch insertion remain pending.
Correction task dispatch, multi-staff/cross-Part/Measure split-merge, general
FRAT population, shared-support image admission and complete cleaner/epilog
remain partial. Completion flags stay false; full non-UI goal is unchanged.
