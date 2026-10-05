#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Convert original page-state observations into declared-context native CLI input."""
import json,sys
from pathlib import Path
def request(o):
 return {'schema':'audiveris-native.stem-head-plan-request.v1','systems':[{'system':s['system'],'system_bounds':s['system_bounds'],'vicinity_margin':s['vicinity_margin'],'beam_ids':s['ordered_beams'],'beams':s['beams'],'beam_bounds':s['beam_bounds'],'groups':s['groups'],'cases':[{k:c[k]for k in ('head_bounds','head','min_distance','small_head')}for c in s['cases']]}for s in o['systems']]}
if __name__=='__main__':Path(sys.argv[2]).write_text(json.dumps(request(json.loads(Path(sys.argv[1]).read_text())),separators=(',',':'))+'\n')
