// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealDotsFactoryImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealDotsFactoryImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static Object field(Object o,String name)throws Exception {for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{var f=k.getDeclaredField(name);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static String articulations(Object sys)throws Exception {
 Object sig=call(sys,"getSig");List<String> rows=new ArrayList<>();for(Object inter:(List<?>)call(sig,"inters",c("sig.inter.ArticulationInter"))) {
 List<String> edges=new ArrayList<>();for(Object rel:(Set<?>)call(sig,"edgesOf",inter))if(rel.getClass().getSimpleName().equals("ChordArticulationRelation")){
 Object ch=call(sig,"getEdgeSource",rel);edges.add("{\"chord_bounds\":"+rect(call(ch,"getBounds"))+",\"grade\":"+call(rel,"getGrade")+",\"dx\":"+call(rel,"getDx")+",\"dy\":"+call(rel,"getDy")+"}");}
 rows.add("{\"glyph\":"+glyph(call(inter,"getGlyph"))+",\"shape\":\""+call(inter,"getShape")+"\",\"abnormal\":"+call(inter,"isAbnormal")+",\"relations\":["+String.join(",",edges)+"]}");
 }return "["+String.join(",",rows)+"]";
 }
 static String articulationLookup(Object sys,Object factory,Object dot)throws Exception {
 Object g=call(dot,"getGlyph"),shape=c("glyph.Shape").getField("STACCATO").get(null),a=make("sig.inter.ArticulationInter",g,shape,Double.valueOf(.8));
 Object link=call(a,"lookupLink",call(factory,"getSystemHeadChords"),call(sys,"getProfile"));
 if(link==null)return "null";Object chord=field(link,"partner"),r=field(link,"relation");
 return "{\"chord_bounds\":"+rect(call(chord,"getBounds"))+",\"grade\":"+call(r,"getGrade")+",\"dx\":"+call(r,"getDx")+",\"dy\":"+call(r,"getDy")+",\"source_ratio\":"+call(r,"getSourceRatio")+",\"target_ratio\":"+call(r,"getTargetRatio")+"}";
 }
 static String artState(Object sig,Object a)throws Exception {
 List<String> edges=new ArrayList<>();if(call(a,"getSig")!=null)for(Object r:(Set<?>)call(sig,"edgesOf",a))if(r.getClass().getSimpleName().equals("ChordArticulationRelation")){
 Object chord=call(sig,"getEdgeSource",r);edges.add("{\"chord_bounds\":"+rect(call(chord,"getBounds"))+",\"grade\":"+call(r,"getGrade")+",\"dx\":"+call(r,"getDx")+",\"dy\":"+call(r,"getDy")+",\"source_ratio\":"+call(r,"getSourceRatio")+",\"target_ratio\":"+call(r,"getTargetRatio")+",\"manual\":"+call(r,"isManual")+"}");}
 return "{\"registered\":"+(call(a,"getSig")!=null)+",\"manual\":"+call(a,"isManual")+",\"abnormal\":"+call(a,"isAbnormal")+",\"relations\":["+String.join(",",edges)+"]}";
 }
 static String artControls(Object sys,Object factory)throws Exception {
 Object sig=call(sys,"getSig");List<String> rows=new ArrayList<>();for(Object observed:new ArrayList<>((List<?>)call(sig,"inters",c("sig.inter.ArticulationInter")))){
 Object glyph=call(observed,"getGlyph"),shape=call(observed,"getShape"),a=make("sig.inter.ArticulationInter",glyph,shape,call(observed,"getGrade"));String plain=artState(sig,a);Object link=call(a,"lookupLink",call(factory,"getSystemHeadChords"),call(sys,"getProfile"));
 if(link==null)throw new IllegalStateException("observed articulation lost lookup");call(sig,"addVertex",a);String added=artState(sig,a);call(link,"applyTo",a);String linked=artState(sig,a);List<String> removals=new ArrayList<>();for(Object r:new ArrayList<>((Set<?>)call(sig,"edgesOf",a))){call(sig,"removeEdge",r);removals.add(artState(sig,a));}
 rows.add("{\"glyph\":"+glyph(glyph)+",\"shape\":\""+shape+"\",\"grade\":"+call(observed,"getGrade")+",\"plain\":"+plain+",\"added\":"+added+",\"linked\":"+linked+",\"removed_states\":["+String.join(",",removals)+"]}");call(a,"remove");
 }return "["+String.join(",",rows)+"]";
 }
 static String lookup(Object sys,Object factory,Object dot)throws Exception {
 Object glyph=call(dot,"getGlyph"),aug=make("sig.inter.AugmentationDotInter",glyph,Double.valueOf(.8));Object link=call(aug,"lookupHeadLink",call(factory,"getSystemHeadChords"),sys,call(sys,"getProfile"));
 if(link==null)return "null";Object head=field(link,"partner"),rel=field(link,"relation");
 return "{\"head_bounds\":"+rect(call(head,"getBounds"))+",\"head_staff\":"+call(call(head,"getStaff"),"getId")+",\"grade\":"+call(rel,"getGrade")+",\"dx\":"+call(rel,"getDx")+",\"dy\":"+call(rel,"getDy")+"}";
 }
 static String candidates(Object sys,Object factory)throws Exception {
 Object df=field(factory,"dotFactory");List<String> rows=new ArrayList<>();for(Object dot:(List<?>)field(df,"dots"))rows.add("{\"glyph\":"+glyph(call(dot,"getGlyph"))+",\"grade\":"+call(dot,"getGrade")+",\"head_lookup\":"+lookup(sys,factory,dot)+",\"articulation_lookup\":"+articulationLookup(sys,factory,dot)+"}");return "["+String.join(",",rows)+"]";
 }
 static String dots(Object sys)throws Exception {
 Object sig=call(sys,"getSig");List<String> rows=new ArrayList<>();for(Object inter:(List<?>)call(sig,"inters",c("sig.inter.AugmentationDotInter"))) {
 Object staff=call(inter,"getStaff");List<String> edges=new ArrayList<>();for(Object rel:(Set<?>)call(sig,"edgesOf",inter))if(!rel.getClass().getSimpleName().equals("Exclusion")){
 Object other=call(sig,"getOppositeInter",inter,rel);edges.add("{\"kind\":\""+rel.getClass().getSimpleName()+"\",\"other_bounds\":"+rect(call(other,"getBounds"))+",\"grade\":"+call(rel,"getGrade")+"}");}
 rows.add("{\"glyph\":"+glyph(call(inter,"getGlyph"))+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"manual\":"+call(inter,"isManual")+",\"abnormal\":"+call(inter,"isAbnormal")+",\"relations\":["+String.join(",",edges)+"]}");
 }return "["+String.join(",",rows)+"]";
 }
 public void process(){try {
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");Map<Object,List<?>> optionals=new LinkedHashMap<>();call(make("sheet.symbol.SymbolsFilter",sheet),"process",optionals);List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")) {
   Object factory=make("sheet.symbol.InterFactory",sys),builder=make("sheet.symbol.SymbolsBuilder",sys,factory);call(builder,"buildSymbols",optionals);String arts=articulations(sys);String artControls=artControls(sys,factory);String controls=candidates(sys,factory);String before=dots(sys);call(factory,"lateChecks");String after=dots(sys);
   systems.add("{\"articulation_controls\":"+artControls+",\"articulations\":"+arts+",\"system\":"+call(sys,"getId")+",\"profile\":"+call(sys,"getProfile")+",\"controls\":"+controls+",\"before_late_checks\":"+before+",\"after_late_checks\":"+after+"}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged image actual CHORDS then SymbolsFilter, SymbolsBuilder with actual InterFactory; actual DotFactory instant/late construction and controlled candidate head lookup before LINKS, no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
