// Copyright © Audiveris 2026. Test observer: native-port contributors, 2026.
// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.lang.reflect.*;
public class GridCompleteProbe extends GridProbe{
 private final Object observed;public GridCompleteProbe(Book b,SortedSet<Integer> ids){super(b,ids);observed=b;}
 static void set(Object o,String name,Object value)throws Exception{for(Class<?> c=o.getClass();c!=null;c=c.getSuperclass())try{Field f=c.getDeclaredField(name);f.setAccessible(true);f.set(o,value);return;}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static String snapshot(Object sheet,String phase)throws Exception{
  List<String>staffs=new ArrayList<>();for(Object staff:(List<?>)call(call(sheet,"getStaffManager"),"getStaves")){
   List<String>lines=new ArrayList<>();for(Object line:(List<?>)call(staff,"getLines")){
    List<Object>ids=new ArrayList<>();for(Object s:(Collection<?>)call(line,"getMembers"))ids.add(call(s,"getId"));
    lines.add("{\"points\":"+points(get(line,"points"))+",\"section_ids\":"+ids+",\"thickness\":"+call(line,"getThickness")+"}");
   }
   staffs.add("{\"id\":"+call(staff,"getId")+",\"left\":"+call(staff,"getAbscissa",Class.forName("org.audiveris.omr.util.HorizontalSide").getField("LEFT").get(null))+",\"right\":"+call(staff,"getAbscissa",Class.forName("org.audiveris.omr.util.HorizontalSide").getField("RIGHT").get(null))+",\"lines\":["+String.join(",",lines)+"]}");
  }return "{\"phase\":\""+phase+"\",\"staffs\":["+String.join(",",staffs)+"]}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(observed,"getStubs")).get(0),"getSheet");Object builder=Class.forName("org.audiveris.omr.sheet.grid.GridBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet")).newInstance(sheet);Object lr=get(builder,"linesRetriever"),bars=get(builder,"barsRetriever"),graph=get(bars,"peakGraph");
  call(lr,"createBothLags");call(lr,"retrieveLines");call(lr,"addShortSections");List<String>barPhases=new ArrayList<>();List<String>linePhases=new ArrayList<>();
  call(graph,"buildSystems");
  for(String phase:List.of("buildColumns","detectStartColumns","purgePartialColumns","purgeTooLeft","detectBracePortions","buildBraces","purgeLeftOfBraces","verifyLinesRoot","detectBracketEnds","detectBracketMiddles","purgeLeftPeaks","purgeUnalignedBars","purgeExtendingPeaks","refineRightEnds","purgeCClefs","partitionWidths","createInters","createConnectionInters","groupBarlines","recordBars","createGroups","createParts","contextualize")){
   call(bars,phase);barPhases.add(snapshot(sheet,phase));
  }
  Object pic=call(sheet,"getPicture"),binary=call(pic,"getSource",Class.forName("org.audiveris.omr.sheet.Picture$SourceKey").getField("BINARY").get(null));set(lr,"binaryBuffer",binary);
  for(String phase:List.of("defineEndPoints","includeDiscardedFilaments","fillHoles")){call(lr,phase);linePhases.add(snapshot(sheet,phase));}
  List<Object>thick=new ArrayList<>(),thin=new ArrayList<>();call(lr,"dispatchHorizontalSections",thick,thin);
  call(lr,"includeSections",thick);linePhases.add(snapshot(sheet,"includeThick"));call(lr,"includeSections",thin);linePhases.add(snapshot(sheet,"includeThin"));
  for(String phase:List.of("polishCurvatures","fillHoles","includeStickers","fillHoles")){call(lr,phase);linePhases.add(snapshot(sheet,phase));}
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"bar_phases\":["+String.join(",",barPhases)+"],\"line_phases\":["+String.join(",",linePhases)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
