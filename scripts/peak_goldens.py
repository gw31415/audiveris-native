#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate deterministic HiLoPeakFinder goldens from compiled unmodified Java."""
import hashlib,random,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];work=ROOT/'evidence/generated/java-scale';jdk=Path(sys.argv[1]).resolve()
source='''import java.util.*;import org.audiveris.omr.math.*;
public class PeakOracle {public static void main(String[] args){Scanner s=new Scanner(System.in);while(s.hasNextInt()){int n=s.nextInt(),d=s.nextInt();IntegerFunction f=new IntegerFunction(0,n-1);for(int i=0;i<n;i++)f.setValue(i,s.nextInt());List<Range> peaks=new HiLoPeakFinder("test",f).findPeaks(1,d,.03);System.out.println(String.join(";",peaks.stream().map(p->p.min+","+p.main+","+p.max).toList()));}}}'''
(work/'PeakOracle.java').write_text(source)
subprocess.run([str(jdk/'bin/javac'),'-cp',str(work/'classes'),'-d',str(work/'classes'),str(work/'PeakOracle.java')],check=True)
cases=[([0,0,0,0],0),([0,4,8,4,0,0],1),([0,8,8,8,0,0],1),([0,8,0,0,8,0,0],1),([0,8,0],1),([0,9,4,0,0,9,4,0,0],1),([0,1,0,0],0),([0,0,0,1],0)]
r=random.Random(20261002)
for i in range(120):
 values=[0]+[r.randrange(0,101) if r.random()>.25 else 0 for _ in range(r.randrange(5,40))]+[0,0]
 cases.append((values,round(sum(values)*.025)))
request='\n'.join(f'{len(v)} {d} '+' '.join(map(str,v)) for v,d in cases)+'\n'
answer=subprocess.check_output([str(jdk/'bin/java'),'-Xmx64m','-cp',str(work/'classes'),'PeakOracle'],input=request.encode()).decode().splitlines()
assert len(answer)==len(cases)
path=ROOT/'fixtures/scale/peak-goldens.txt'
path.write_text('# Pinned unmodified Java HiLoPeakFinder; minValue=1, minGainRatio=.03; seed=20261002\n'+'\n'.join(f'{d}|'+','.join(map(str,v))+'|'+a for (v,d),a in zip(cases,answer))+'\n')
print('Generated',len(cases),'Java peak goldens;',hashlib.sha256(path.read_bytes()).hexdigest())
