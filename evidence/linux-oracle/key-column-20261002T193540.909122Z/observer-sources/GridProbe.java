// Copyright © Audiveris 2026. Test instrumentation: 2026 native-port contributors.
// SPDX-License-Identifier: AGPL-3.0-or-later
// Calls unmodified official classes after actual raster LOAD/BINARY/SCALE.
import org.audiveris.omr.step.RunClass;
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
public class GridProbe extends RunClass {
 private final Object observedBook;
 public GridProbe(Book b, SortedSet<Integer> ids){super(b,ids);observedBook=b;}
 static Object call(Object o,String name,Object...args)throws Exception{for(Method m:o.getClass().getMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length)return m.invoke(o,args);for(Class<?> c=o.getClass();c!=null;c=c.getSuperclass())for(Method m:c.getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);return m.invoke(o,args);}throw new NoSuchMethodException(name);}
 static Object get(Object o,String name)throws Exception{for(Class<?> c=o.getClass();c!=null;c=c.getSuperclass()){try{Field f=c.getDeclaredField(name);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException e){}}throw new NoSuchFieldException(name);}
 public void process(){try{
  Object sheet=call(((List<?>)call(observedBook,"getStubs")).get(0),"getSheet");Object builder=Class.forName("org.audiveris.omr.sheet.grid.GridBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet")).newInstance(sheet);Object linesRetriever=get(builder,"linesRetriever");
  call(linesRetriever,"createBothLags");call(linesRetriever,"retrieveLines");call(linesRetriever,"addShortSections");
  List<String>result=new ArrayList<>();
  for(Object staff:(List<?>)call(call(sheet,"getStaffManager"),"getStaves")){
   Object p=Class.forName("org.audiveris.omr.sheet.grid.StaffProjector").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet"),Class.forName("org.audiveris.omr.sheet.Staff"),Class.forName("org.audiveris.omr.sheet.grid.PeakGraph")).newInstance(sheet,staff,null);call(p,"process");
   Object proj=get(p,"projection");Object params=get(p,"params");
   List<String>blanks=new ArrayList<>();for(Object b:(List<?>)get(p,"allBlanks"))blanks.add("["+get(b,"start")+","+get(b,"stop")+"]");
   List<String>peakGrades=new ArrayList<>();for(Object peak:(List<?>)call(p,"getPeaks"))peakGrades.add(call(call(peak,"getImpacts"),"getGrade").toString());
   List<String>peaks=new ArrayList<>();for(Object peak:(List<?>)call(p,"getPeaks"))peaks.add("["+call(peak,"getStart")+","+call(peak,"getStop")+","+call(peak,"getTop")+","+call(peak,"getBottom")+"]");
   List<String>lines=new ArrayList<>();for(Object line:(List<?>)call(staff,"getLines"))lines.add("{\"thickness\":"+call(line,"getThickness")+",\"points\":"+points(get(line,"points"))+"}");
   List<Integer>values=new ArrayList<>();for(int x=0;x<(int)call(sheet,"getWidth");x++)values.add((Integer)call(proj,"getValue",x));
   Map<?,?>ending=(Map<?,?>)get(p,"endingBlanks");List<String>ends=new ArrayList<>();for(var e:ending.entrySet())ends.add("\""+e.getKey()+"\":["+get(e.getValue(),"start")+","+get(e.getValue(),"stop")+"]");
   result.add("{\"staff_id\":"+call(staff,"getId")+",\"left\":"+call(staff,"getAbscissa",Class.forName("org.audiveris.omr.util.HorizontalSide").getField("LEFT").get(null))+",\"right\":"+call(staff,"getAbscissa",Class.forName("org.audiveris.omr.util.HorizontalSide").getField("RIGHT").get(null))+",\"lines\":["+String.join(",",lines)+"],\"projection\":"+values+",\"derivative_threshold\":"+get(p,"derivativeThreshold")+",\"blank_threshold\":"+get(params,"blankThreshold")+",\"lines_threshold\":"+get(params,"linesThreshold")+",\"chunk_threshold\":"+get(params,"chunkThreshold")+",\"blanks\":["+String.join(",",blanks)+"],\"ending_blanks\":{"+String.join(",",ends)+"},\"peaks\":["+String.join(",",peaks)+"],\"peak_grades\":["+String.join(",",peakGrades)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"["+String.join(",",result)+"]");
  Object graph=get(get(builder,"barsRetriever"),"peakGraph");call(graph,"findBarPeaks");call(graph,"buildBarSticks");call(graph,"detectCurvedPeaks");
  List<String>sticksResult=new ArrayList<>();
  for(Object projector:(List<?>)get(graph,"projectors")){
   Object staff=call(projector,"getStaff");List<String>sticks=new ArrayList<>();
   for(Object peak:(List<?>)call(projector,"getPeaks")){
    Object fil=call(peak,"getFilament");List<String>members=new ArrayList<>();
    for(Object section:(Collection<?>)call(fil,"getMembers")){java.awt.Rectangle b=(java.awt.Rectangle)call(section,"getBounds");members.add("{\"orientation\":\""+call(section,"getOrientation").toString().toLowerCase()+"\",\"id\":"+call(section,"getId")+",\"bounds\":["+b.x+","+b.y+","+b.width+","+b.height+"],\"weight\":"+call(section,"getWeight")+"}");}
    double radius=(Double)call(fil,"getMeanCurvature");
    sticks.add("{\"peak\":["+call(peak,"getStart")+","+call(peak,"getStop")+","+call(peak,"getTop")+","+call(peak,"getBottom")+"],\"grade\":"+call(call(peak,"getImpacts"),"getGrade")+",\"points\":"+points(get(fil,"points"))+",\"brace\":"+(radius<10*(int)call(call(sheet,"getScale"),"getInterline"))+",\"mean_radius\":"+(Double.isFinite(radius)?Double.toString(radius):"null")+",\"sections\":["+String.join(",",members)+"]}");
   }
   sticksResult.add("{\"staff_id\":"+call(staff,"getId")+",\"sticks\":["+String.join(",",sticks)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")+".sticks.json"),"["+String.join(",",sticksResult)+"]");
  List<String>graphStages=new ArrayList<>();
  for(String phase:List.of("findAllAlignments","findConnections","splitMergedGroups","purgeAlignments")){
   call(graph,phase);List<String>edges=new ArrayList<>();
   for(Object edge:(Set<?>)call(graph,"edgeSet")){Object top=get(edge,"topPeak"),bot=get(edge,"bottomPeak");edges.add("{\"top\":["+call(call(top,"getStaff"),"getId")+","+call(top,"getStart")+","+call(top,"getStop")+"],\"bottom\":["+call(call(bot,"getStaff"),"getId")+","+call(bot,"getStart")+","+call(bot,"getStop")+"],\"connection\":"+edge.getClass().getSimpleName().equals("BarConnection")+",\"grade\":"+call(edge,"getGrade")+"}");}
   graphStages.add("{\"phase\":\""+phase+"\",\"edges\":["+String.join(",",edges)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")+".graph.json"),"{\"stages\":["+String.join(",",graphStages)+"],\"system_tops\":"+Arrays.toString((Object[])call(graph,"getSystemTops"))+"}");


 }catch(Exception e){throw new RuntimeException(e);}}
 static String points(Object raw){List<String>p=new ArrayList<>();for(Object v:(List<?>)raw){var x=(java.awt.geom.Point2D)v;p.add("["+x.getX()+","+x.getY()+"]");}return "["+String.join(",",p)+"]";}
}
