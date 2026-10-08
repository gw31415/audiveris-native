// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealSymbolAltersImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealSymbolAltersImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String relations(Object sig,Object inter)throws Exception {
  List<String> rows=new ArrayList<>();for(Object rel:(Set<?>)call(sig,"edgesOf",inter))if(rel.getClass().getSimpleName().equals("AlterHeadRelation")) {
   Object head=call(sig,"getEdgeTarget",rel);rows.add("{\"head_bounds\":"+rect(call(head,"getBounds"))+",\"head_staff\":"+call(call(head,"getStaff"),"getId")+",\"grade\":"+call(rel,"getGrade")+",\"dx\":"+call(rel,"getDx")+",\"dy\":"+call(rel,"getDy")+",\"manual\":"+call(rel,"isManual")+",\"source_ratio\":"+call(rel,"getSourceRatio")+",\"target_ratio\":"+call(rel,"getTargetRatio")+"}");
  }return "["+String.join(",",rows)+"]";
 }
 static String state(Object sig,Object inter)throws Exception {
  return "{\"registered\":"+(call(inter,"getSig")!=null)+",\"manual\":"+call(inter,"isManual")+",\"abnormal\":"+call(inter,"isAbnormal")+",\"relations\":"+(call(inter,"getSig")==null?"[]":relations(sig,inter))+"}";
 }
 static String alters(Object sys)throws Exception {
  Object sig=call(sys,"getSig");List<String> rows=new ArrayList<>();
  for(Object inter:(List<?>)call(sig,"inters",c("sig.inter.AlterInter"))) {
   if(!inter.getClass().getSimpleName().equals("AlterInter"))continue;
   Object staff=call(inter,"getStaff");
   rows.add("{\"id\":"+call(inter,"getId")+",\"shape\":\""+call(inter,"getShape")+"\",\"bounds\":"+rect(call(inter,"getBounds"))+",\"staff\":"+call(staff,"getId")+",\"grade\":"+call(inter,"getGrade")+",\"pitch\":"+call(inter,"getPitch")+",\"measured_pitch\":"+call(inter,"getMeasuredPitch")+",\"state\":"+state(sig,inter)+",\"glyph\":"+glyph(call(inter,"getGlyph"))+"}");
  }
  return "["+String.join(",",rows)+"]";
 }
 static String controls(Object sys,Object factory)throws Exception {
  Object sig=call(sys,"getSig");var headsField=c("sheet.symbol.InterFactory").getDeclaredField("systemHeads");headsField.setAccessible(true);List<?> heads=(List<?>)headsField.get(factory);List<String> rows=new ArrayList<>();
  List<?> original=new ArrayList<>((List<?>)call(sig,"inters",c("sig.inter.AlterInter")));
  for(Object observed:original) {
   if(!observed.getClass().getSimpleName().equals("AlterInter"))continue;
   Object glyph=call(observed,"getGlyph"),shape=call(observed,"getShape"),staff=call(observed,"getStaff");
   var create=c("sig.inter.AlterInter").getMethod("create",c("glyph.Glyph"),c("glyph.Shape"),Double.class,c("sheet.Staff"));Object shadow=create.invoke(null,glyph,shape,call(observed,"getGrade"),staff);
   String plain=state(sig,shadow);call(sig,"addVertex",shadow);String added=state(sig,shadow);call(shadow,"setLinks",heads);String linked=state(sig,shadow);List<String> removals=new ArrayList<>();
   List<?> edges=new ArrayList<>((Set<?>)call(sig,"edgesOf",shadow));for(Object rel:edges)if(rel.getClass().getSimpleName().equals("AlterHeadRelation")){call(sig,"removeEdge",rel);removals.add(state(sig,shadow));}
   rows.add("{\"shape\":\""+shape+"\",\"glyph\":"+glyph(glyph)+",\"plain\":"+plain+",\"added\":"+added+",\"linked\":"+linked+",\"removed_states\":["+String.join(",",removals)+"]}");call(shadow,"remove");
  }return "["+String.join(",",rows)+"]";
 }
 public void process(){try {
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");Map<Object,List<?>> optionals=new LinkedHashMap<>();call(make("sheet.symbol.SymbolsFilter",sheet),"process",optionals);List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")) {
   Object factory=make("sheet.symbol.InterFactory",sys),builder=make("sheet.symbol.SymbolsBuilder",sys,factory);call(builder,"buildSymbols",optionals);String before=alters(sys);call(factory,"lateChecks");String after=alters(sys);String controls=controls(sys,factory);
   systems.add("{\"system\":"+call(sys,"getId")+",\"profile\":"+call(sys,"getProfile")+",\"controls\":"+controls+",\"before_late_checks\":"+before+",\"after_late_checks\":"+after+"}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged image actual CHORDS then SymbolsFilter, SymbolsBuilder with actual InterFactory; actual AlterInter construction before LINKS, no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
