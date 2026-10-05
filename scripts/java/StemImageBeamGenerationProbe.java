// SPDX-License-Identifier: AGPL-3.0-or-later
// Original beam constructors and findLinker; no native outputs or injected page state.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.geom.*;import java.lang.reflect.*;
public class StemImageBeamGenerationProbe extends StemImageBeamTargetsProbe {
 private final Object book;public StemImageBeamGenerationProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static Object fresh(Object b,Object retr)throws Exception{return Class.forName("org.audiveris.omr.sheet.stem.BeamLinker").getConstructor(Class.forName("org.audiveris.omr.sig.inter.AbstractBeamInter"),Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever")).newInstance(b,retr);}
 static String bLinker(Object l)throws Exception{String base=StemImageBeamTargetsProbe.bLinker(l);List<String>ds=new ArrayList<>();for(Object d:((Map<?,?>)get(l,"vLinkers")).keySet())ds.add("\""+d+"\"");return base.substring(0,base.length()-1)+",\"directions\":["+String.join(",",ds)+"]}";}
 static String links(Object bl)throws Exception{List<String>ls=new ArrayList<>();for(Object l:(List<?>)get(bl,"allBLinkers"))ls.add(bLinker(l));return "["+String.join(",",ls)+"]";}
 static String purge(Object bl,Object p,String name,int[][]boxes,int dx)throws Exception{List<Object>gs=new ArrayList<>();for(int[]b:boxes)gs.add(glyph(b[0],b[1],b[2],b[3],"VERTICAL"));String input=glyphs(gs);set(p,"minBeamStemsDx",dx);call(bl,"purgeSeeds",gs);return "{\"name\":\""+name+"\",\"median\":"+line(get(bl,"median"))+",\"min_dx\":"+dx+",\"input\":"+input+",\"selected\":"+glyphs(gs)+"}";}
 public void process(){try{
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),first=null,params=null;List<String>ss=new ArrayList<>(),controls=new ArrayList<>(),nearest=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){Object retr=inspect(sys,sheet),p=get(retr,"params");List<String>bs=new ArrayList<>();for(Object b:(List<?>)get(retr,"systemBeams")){Object actual=call(b,"getLinker"),base=fresh(b,retr);if(first==null){first=base;params=p;}List<Object>eligible=new ArrayList<>();Object area=call(base,"getSeedArea");for(Object g:(Collection<?>)get(base,"neighborSeeds"))if(((Area)area).intersects((java.awt.Rectangle)call(g,"getBounds")))eligible.add(g);eligible.sort((a,c)->{try{return Double.compare(((Point2D)callStatic("math.LineUtil","intersection",call(a,"getCenterLine"),get(base,"median"))).getX(),((Point2D)callStatic("math.LineUtil","intersection",call(c,"getCenterLine"),get(base,"median"))).getX());}catch(Exception e){throw new RuntimeException(e);}});String before=glyphs(eligible);call(base,"purgeSeeds",eligible);
 bs.add("{\"beam\":"+beam(b)+",\"seed_area_path\":"+path(area)+",\"admitted_seeds\":"+before+",\"purged_seeds\":"+glyphs(eligible)+",\"stumps\":"+glyphs((Collection<?>)get(base,"stumps"))+",\"base_blinkers\":"+links(base)+",\"final_blinkers\":"+links(actual)+"}");}
 ss.add("{\"system\":"+call(sys,"getId")+",\"main_stem_thickness\":"+get(p,"mainStemThickness")+",\"max_stem_thickness\":"+get(p,"maxStemThickness")+",\"min_beam_stems_dx\":"+get(p,"minBeamStemsDx")+",\"max_beam_seed_dx\":"+get(p,"maxBeamSeedDx")+",\"max_beam_linker_dx\":"+get(p,"maxBeamLinkerDx")+",\"min_beam_stump_dy\":"+get(p,"minBeamStumpDy")+",\"beam_catalog\":["+String.join(",",bs)+"]}");}
 // Direct original purge conditions use fresh glyphs, including touching and disjoint line distances.
 controls.add(purge(first,params,"empty",new int[][]{},11));
 controls.add(purge(first,params,"equal-height-overlap",new int[][]{{490,370,1,30},{495,375,1,30}},11));
 controls.add(purge(first,params,"taller-second",new int[][]{{490,375,1,20},{495,370,1,30}},11));
 controls.add(purge(first,params,"touching-disjoint-distance",new int[][]{{490,360,1,10},{495,370,1,10}},11));
 for(int dx:new int[]{4,5,6})controls.add(purge(first,params,"dx-"+dx,new int[][]{{490,370,1,30},{495,375,1,31}},dx));
 controls.add(purge(first,params,"removal-skips-next-like-original",new int[][]{{490,370,1,30},{492,370,1,20},{494,370,1,20}},11));
 controls.add(purge(first,params,"replace-first-then-revisit",new int[][]{{490,370,1,20},{492,370,1,30},{494,370,1,40}},11));
 // Isolated findLinker controls. Each query starts from the same original constructor catalog.
 Object beam=get(first,"beam"),retr=get(first,"retriever");Object ref=call(((List<?>)get(first,"allBLinkers")).get(0),"getReferencePoint");double x=((Point2D)ref).getX();set(params,"maxBeamLinkerDx",1);
 for(double q:new double[]{x,Math.nextDown(x+1),x+1,Math.nextUp(x+1),x+5}){Object bl=fresh(beam,retr);String before=links(bl);Line2D ln=new Line2D.Double(q,0,q,2000);Object chosen=call(bl,"findLinker",ln);nearest.add("{\"line\":"+line(ln)+",\"median\":"+line(get(bl,"median"))+",\"max_dx\":1,\"before\":"+before+",\"selected\":"+bLinker(chosen)+",\"after\":"+links(bl)+"}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"original_phase\":\"inspectStems_before_linkStems_and_finalizeStems\",\"systems\":["+String.join(",",ss)+"],\"purge_controls\":["+String.join(",",controls)+"],\"nearest_controls\":["+String.join(",",nearest)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
 static Object callStatic(String name,String method,Object...args)throws Exception{Class<?>c=Class.forName("org.audiveris.omr."+name);for(Method m:c.getDeclaredMethods())if(m.getName().equals(method)&&m.getParameterCount()==args.length){boolean ok=true;for(int i=0;i<args.length;i++)if(!m.getParameterTypes()[i].isInstance(args[i]))ok=false;if(ok){m.setAccessible(true);return m.invoke(null,args);}}throw new NoSuchMethodException(method);}
}
