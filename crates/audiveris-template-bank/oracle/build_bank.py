"""Package the already-generated official assets. Development only; no downloads."""
from pathlib import Path
import json,hashlib,zipfile,os
root=Path(__file__).resolve().parent.parent
app=Path(os.environ['AUDIVERIS_CONTENTS'])
sha=lambda b:hashlib.sha256(b).hexdigest()
jar=(app/'app/audiveris.jar').read_bytes()
assert sha(jar)=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
data=json.loads((root/'oracle/generated/assets.json').read_text())
cal=json.loads((root/'oracle/generated/calibration.json').read_text())
coverage=json.loads((root/'oracle/run/coverage.json').read_text())
execution=json.loads((root/'oracle/run/execution.json').read_text());assert execution['exit']==0
required=set()
for sheet in coverage['sheets']:
 assert sheet['family']=='Bravura'
 for staff in sheet['staves']:
  if staff['tablature']:continue
  assert not staff['drum'], 'Drum pitch-dependent shape coverage needs separate enumeration'
  required.update((shape,staff['point_size']) for shape in sheet['shapes']['All'])
assets=[]
for a in data['assets']:
 assets.append({'key':{'shape':a['shape'],'point_size':a['point_size']},'width':a['width'],'height':a['height'],'fat_bounds':[int(v) for v in a['fat_bounds']],'slim_bounds':[int(v) for v in a['slim_bounds']],'anchors':a['anchors'],'points':[{'x':v[0],'y':v[1],'d':v[2]} for v in a['points']]})
assert {(a['key']['shape'],a['key']['point_size']) for a in assets}==required
with zipfile.ZipFile(app/'app/audiveris.jar') as z:
 font=z.read('res/Bravura.otf');model=z.read('res/basic-classifier.zip')
assert sha(font)==data['font_sha256']
payload=(json.dumps({'schema':1,'templates':assets,'calibration':cal},separators=(',',':'),allow_nan=False)+'\n').encode()
manifest={'schema':1,'bank_id':'audiveris-5.11.0-bravura-1.392-heads51-v1','upstream_version':'5.11.0','upstream_commit':data['commit'],'font_family':'Bravura','font_version':'1.392','font_sha256':sha(font),'model_name':'basic-classifier.zip','model_sha256':sha(model),'coverage_input_sha256':execution['input_sha256'],'generator':'BankAssetOracle-v1; official JAR; Zulu25.0.3; macOS arm64; default TemplateFactory constants','asset_sha256':sha(payload),'keys':[a['key'] for a in assets],'calibration_sizes':[m['point_size'] for m in cal]}
mb=(json.dumps(manifest,indent=2)+'\n').encode()
(root/'assets/manifest.json').write_bytes(mb);(root/'assets/templates.json').write_bytes(payload)
trusted={'manifest_sha256':sha(mb),'upstream_commit':manifest['upstream_commit'],'font_sha256':manifest['font_sha256'],'model_sha256':manifest['model_sha256'],'coverage_input_sha256':manifest['coverage_input_sha256']}
(root/'src/builtin_identity.rs').write_text("pub const BUILTIN_TRUST: Trust<'static> = Trust {\n"+''.join(f'    {k}: "{v}",\n' for k,v in trusted.items())+'};\n')
(root/'oracle/asset-build.json').write_text(json.dumps({'manifest_sha256':sha(mb),'payload_sha256':sha(payload),'manifest_bytes':len(mb),'payload_bytes':len(payload),'total_points':sum(len(a['points']) for a in assets),'generation_input_sha256':sha((root/'oracle/generated/assets.json').read_bytes()),'official_jar_sha256':sha(jar),'input_sha256':execution['input_sha256']},indent=2)+'\n')
print(len(assets),'templates;',len(mb)+len(payload),'bank bytes')
