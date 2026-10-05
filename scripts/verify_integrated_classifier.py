#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Validate the root CLI's parsing/serialization against the imported Java golden."""
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];exe=ROOT/(sys.argv[1] if len(sys.argv)>1 else 'target/debug/audiveris-native');crate=ROOT/'crates/audiveris-classifier';work=ROOT/'evidence/generated/classifier-cli';work.mkdir(parents=True,exist_ok=True)
golden_path=crate/'tests/fixtures/java-5.11.0-golden.json';golden=json.loads(golden_path.read_text());assert hashlib.sha256(golden_path.read_bytes()).hexdigest()=='432e7003d8c2b319575e79c023a6f378cd4f18f9092b5ce47fb8aa1d46dfe983'
features=work/'features.json';model=crate/'models/audiveris-5.11.0.json';maximum=0.;count=0
for case in golden['feature_cases']:
 features.write_text(json.dumps(case['raw_features']))
 actual=json.loads(subprocess.check_output([str(exe),'classify-features',str(model),str(features)]))
 assert actual['class_labels']==golden['output_labels']
 assert len(actual['scores'])==149
 for got,expected in zip(actual['scores'],case['scores']):
  delta=abs(got-expected);maximum=max(maximum,delta);assert delta<=1e-12+1e-10*abs(expected);count+=1
 rank=sorted(range(149),key=lambda i:(-actual['scores'][i],i))[:5];assert rank==[v['index'] for v in case['top5']]
for bad in ['[0]','[NaN]','null','['+','.join(['1e308']*110)+']']:
 features.write_text(bad);result=subprocess.run([str(exe),'classify-features',str(model),str(features)],capture_output=True,text=True);assert result.returncode==1 and not result.stdout
report={'cases':136,'cli_scores_compared':count,'max_absolute_score_error':maximum,'absolute_tolerance':1e-12,'relative_tolerance':1e-10,'top5_order_match':True,'invalid_input_cases':4,'root_cli_error_exit':1,'import_commit':'185b0992172197a41cbfa5ec85f335471b01df7b','history_import_commit':'ee5f3ff','golden_sha256':hashlib.sha256(golden_path.read_bytes()).hexdigest(),'image_to_classifier':False,'profile':str(exe.relative_to(ROOT))}
(ROOT/'evidence/classifier-integration.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
