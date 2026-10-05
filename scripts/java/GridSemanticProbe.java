// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent official raster pipeline, observing the unchanged private methods.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;
public class GridSemanticProbe extends GridProbe {
 private final Object observed;
 public GridSemanticProbe(Book b,SortedSet<Integer> ids){super(b,ids);observed=b;}
 static String serifs(Object p)throws Exception{List<String>out=new ArrayList<>();for(String side:List.of("TOP","BOTTOM")){Object f=call(p,side.equals("TOP")?"getTopSerif":"getBottomSerif");if(f==null)continue;List<String>keys=new ArrayList<>();for(Object sec:(Collection<?>)call(f,"getMembers"))keys.add("[\""+call(sec,"getOrientation")+"\","+call(sec,"getId")+"]");out.add("{\"side\":\""+side+"\",\"weight\":"+call(f,"getWeight")+",\"section_keys\":["+String.join(",",keys)+"]}");}return "["+String.join(",",out)+"]";}
 static String state(Object bars,String phase)throws Exception{
  List<String>staffs=new ArrayList<>();
  for(Object pr:(List<?>)get(bars,"projectors")){
   Object staff=call(pr,"getStaff");List<String>peaks=new ArrayList<>();
   for(Object p:(List<?>)call(pr,"getPeaks"))peaks.add("{\"key\":["+call(staff,"getId")+","+call(p,"getStart")+","+call(p,"getStop")+"],\"attributes\":\""+get(p,"attrs")+"\",\"serifs\":"+serifs(p)+"}");
   Class<?> side=Class.forName("org.audiveris.omr.util.HorizontalSide");
   staffs.add("{\"id\":"+call(staff,"getId")+",\"left\":"+call(staff,"getAbscissa",side.getField("LEFT").get(null))+",\"right\":"+call(staff,"getAbscissa",side.getField("RIGHT").get(null))+",\"peaks\":["+String.join(",",peaks)+"]}");
  }return "{\"phase\":\""+phase+"\",\"staffs\":["+String.join(",",staffs)+"]}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(observed,"getStubs")).get(0),"getSheet");
  Object builder=Class.forName("org.audiveris.omr.sheet.grid.GridBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet")).newInstance(sheet);
  Object lr=get(builder,"linesRetriever"),bars=get(builder,"barsRetriever"),graph=get(bars,"peakGraph");
  call(lr,"createBothLags");call(lr,"retrieveLines");call(lr,"addShortSections");call(graph,"buildSystems");
  List<String>phases=new ArrayList<>();
  for(String phase:List.of("buildColumns","detectStartColumns","purgePartialColumns","purgeTooLeft","detectBracePortions","buildBraces","purgeLeftOfBraces","verifyLinesRoot","detectBracketEnds","detectBracketMiddles","purgeLeftPeaks","purgeUnalignedBars","purgeExtendingPeaks","refineRightEnds","purgeCClefs")){
   call(bars,phase);phases.add(state(bars,phase));
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"phases\":["+String.join(",",phases)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
