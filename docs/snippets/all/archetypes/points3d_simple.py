"""Log some very simple points."""

import simplant_lab as rr

rr.init("rerun_example_points3d", spawn=True)

rr.log("points", rr.Points3D([[0, 0, 0], [1, 1, 1]]))
