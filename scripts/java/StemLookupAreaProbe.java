// SPDX-License-Identifier: AGPL-3.0-or-later
// Original buildLuArea invoked on legitimate Laura CLinkers. No native outputs.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class StemLookupAreaProbe extends StemPartsLookupProbe {
 private final Object originalBook;
 public StemLookupAreaProbe(Book b,SortedSet<Integer> ids){super(b,ids);originalBook=b;}
 static String point(Point2D p){return "["+p.getX()+","+p.getY()+"]";}
 static String line(Line2D p){return "["+p.getX1()+","+p.getY1()+","+p.getX2()+","+p.getY2()+"]";}
 static String observe(Object c,Object sys,String mode,Line2D limit)throws Exception {
  Object side=get(c,"this$1"),headLinker=get(side,"this$0"),params=get(headLinker,"params"),head=call(c,"getHead");
  Point2D ref=(Point2D)get(c,"refPt");Object part=call(call(head,"getStaff"),"getPart");
  Object area=call(c,"buildLuArea",limit);
  return "{\"mode\":\""+mode+"\",\"reference\":"+point(ref)+",\"x_direction\":"+get(side,"xDir")+",\"y_direction\":"+get(c,"yDir")+",\"slope\":"+call(call(sys,"getSkew"),"getSlope")+",\"slope_margin\":"+get(params,"slopeMargin")+",\"max_head_in_dx\":"+get(params,"maxHeadInDx")+",\"max_head_out_dx\":"+get(params,"maxHeadOutDx")+",\"part_bounds\":"+rect(call(part,"getAreaBounds"))+",\"limit\":"+(limit==null?"null":line(limit))+",\"area_path\":"+path(area)+",\"theoretical_line\":"+line((Line2D)get(c,"theoLine"))+"}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(originalBook,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
  Object retr=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);call(retr,"process");
  List<String> rows=new ArrayList<>();int count=0;
  for(Object head:(List<?>)get(retr,"systemHeads")){Object linker=call(head,"getLinker");if(linker==null)continue;
   for(String side:new String[]{"LEFT","RIGHT"})for(String vert:new String[]{"TOP","BOTTOM"}){
    Object c=call(linker,"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",vert));if(get(c,"sb")==null)continue;count++;
    Object old=get(c,"theoLine");Point2D ref=(Point2D)get(c,"refPt");int dir=(Integer)get(c,"yDir");
    try{rows.add(observe(c,sys,"part-edge",null));
     double far=ref.getY()+dir*100;rows.add(observe(c,sys,"declared-sloped-limit",new Line2D.Double(ref.getX()-100,far-5,ref.getX()+100,far+5)));
     for(String mode:new String[]{"limit-below","limit-equal","limit-above"}){double y=mode.equals("limit-below")?Math.nextDown(ref.getY()):mode.equals("limit-above")?Math.nextUp(ref.getY()):ref.getY();rows.add(observe(c,sys,mode,new Line2D.Double(0,y,100,y)));}
    }finally{set(c,"theoLine",old);}
   }
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Head CLinker area generation from actual first-system reference points and declared part/line limits; beam target selection and full page STEMS are not claimed\",\"head_builders\":"+count+",\"cases\":["+String.join(",",rows)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
