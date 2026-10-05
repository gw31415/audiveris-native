// SPDX-License-Identifier: AGPL-3.0-or-later
// Legitimate original VLinkers, original geometry method, no native outputs.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class StemBeamLookupAreaProbe extends StemPartsLookupProbe {
 private final Object originalBook;
 public StemBeamLookupAreaProbe(Book b,SortedSet<Integer> ids){super(b,ids);originalBook=b;}
 static String line(Line2D l){return "["+l.getX1()+","+l.getY1()+","+l.getX2()+","+l.getY2()+"]";}
 static String observe(Object c,Object sys,String mode,Line2D limit)throws Exception {
  Object side=get(c,"this$1"),linker=get(side,"this$0"),params=get(linker,"params"),beam=get(linker,"beam");
  Point2D ref=(Point2D)get(side,"refPt");Line2D border=(Line2D)call(beam,"getBorder",get(c,"vSide"));
  int profile=Math.max((Integer)call(beam,"getProfile"),(Integer)call(sys,"getProfile"));
  Object gap=Class.forName("org.audiveris.omr.sig.relation.BeamStemRelation").getMethod("getYGapMaximum",int.class).invoke(null,profile);
  Object scale=get(linker,"scale");int pixels=(Integer)scale.getClass().getMethod("toPixels",Class.forName("org.audiveris.omr.sheet.Scale$Fraction")).invoke(scale,gap);
  List<String>parts=new ArrayList<>();for(Object staff:(List<?>)call(sys,"getStavesAround",call(beam,"getCenter")))parts.add(rect(call(call(staff,"getPart"),"getAreaBounds")));
  Object area=call(c,"buildLuArea",limit);
  return "{\"mode\":\""+mode+"\",\"profile\":"+profile+",\"reference\":["+ref.getX()+","+ref.getY()+"],\"border\":"+line(border)+",\"direction\":"+get(c,"yDir")+",\"slope\":"+call(call(sys,"getSkew"),"getSlope")+",\"slope_margin\":"+get(params,"slopeMargin")+",\"half_lookup_dx\":"+get(params,"halfBeamLuDx")+",\"max_seed_dy_ratio\":"+get(params,"maxBeamSeedDyRatio")+",\"gap_pixels\":"+pixels+",\"system_bounds\":"+rect(call(sys,"getBounds"))+",\"surrounding_part_bounds\":["+String.join(",",parts)+"],\"limit\":"+(limit==null?"null":line(limit))+",\"area_path\":"+path(area)+",\"theoretical_line\":"+line((Line2D)get(c,"theoLine"))+"}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(originalBook,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
  Object retr=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);call(retr,"process");
  List<String>rows=new ArrayList<>();int count=0;
  for(Object beam:(List<?>)get(retr,"systemBeams")){Object linker=call(beam,"getLinker");if(linker==null)continue;
   for(Object side:(List<?>)get(linker,"allBLinkers"))for(Object c:(Collection<?>)call(side,"getHalfLinkers")){
    if(get(c,"sb")==null)continue;count++;Object params=get(linker,"params"),oldLine=get(c,"theoLine");double ratio=(Double)get(params,"maxBeamSeedDyRatio");Point2D ref=(Point2D)get(side,"refPt");int dir=(Integer)get(c,"yDir");double far=ref.getY()+dir*100;Line2D limit=new Line2D.Double(ref.getX()-100,far-5,ref.getX()+100,far+5);
    try{rows.add(observe(c,sys,"part-edge",null));rows.add(observe(c,sys,"declared-sloped-limit",limit));
     for(String mode:new String[]{"ratio-below","ratio-equal","ratio-above","ratio-zero"}){set(params,"maxBeamSeedDyRatio",mode.equals("ratio-below")?Math.nextDown(ratio):mode.equals("ratio-above")?Math.nextUp(ratio):mode.equals("ratio-zero")?0.0:ratio);rows.add(observe(c,sys,mode,limit));}
    }finally{set(params,"maxBeamSeedDyRatio",ratio);set(c,"theoLine",oldLine);}
   }
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Original first-system VLinker geometry with declared borders/part neighborhoods/profile gap pixels; no automatic neighborhood/closer-limit or whole page STEMS claim\",\"beam_builders\":"+count+",\"cases\":["+String.join(",",rows)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
