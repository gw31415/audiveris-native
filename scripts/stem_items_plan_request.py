#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Convert declared original selected source observations, not image discovery."""
import json,sys
from pathlib import Path
def make(o):
 return {'schema':'audiveris-native.stem-items-plan-request.v1','cases':[{'direction':g['direction'],'start_y':g['start_y'],'y_range':g['y_range'],'sources':c['sources'],'gap_map':o['gap_map'],'stem_profile':c['stem_profile'],'max_profile':c['max_profile'],'initial_lengths':{},'skew':0.} for g in o['groups']for c in g['cases']]}
if __name__=='__main__':Path(sys.argv[2]).write_text(json.dumps(make(json.loads(Path(sys.argv[1]).read_text())),separators=(',',':'))+'\n')
