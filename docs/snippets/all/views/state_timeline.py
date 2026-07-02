# Use a blueprint to show a StateTimelineView.

import rerun.blueprint as rrb

import simplant_lab as rr

rr.init("rerun_example_state_timeline", spawn=True)

rr.set_time("step", sequence=0)
rr.log("door", rr.StateChange(state="open"))

rr.set_time("step", sequence=1)
rr.log("door", rr.StateChange(state="closed"))

rr.set_time("step", sequence=2)
rr.log("door", rr.StateChange(state="open"))

# Create a state timeline view to display the state transitions.
blueprint = rrb.Blueprint(
    rrb.StateTimelineView(
        origin="/",
        name="State Transitions",
    ),
    collapse_panels=True,
)

rr.send_blueprint(blueprint)
