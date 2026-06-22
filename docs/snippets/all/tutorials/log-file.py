import sys

import simplant_lab as rr

rr.init("rerun_example_log_file", spawn=True)

rr.log_file_from_path(sys.argv[1])
