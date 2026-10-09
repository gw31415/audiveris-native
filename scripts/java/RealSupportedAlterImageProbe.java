// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealSupportedAlterImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealSupportedAlterImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String supports(Object sig,Object alter)throws Exception {
 List<String> rows=new ArrayList<>();for(Object rel:(Set<?>)call(sig,"edgesOf",alter))if(rel.getClass().getSimpleName().equals("AlterHeadRelation")){Object head=call(sig,"getEdgeTarget",rel);rows.add("{\"head_bounds\":"+rect(call(head,"getBounds"))+",\"head_staff\":"+call(call(head,"getStaff"),"getId")+",\"head_grade\":"+call(head,"getGrade")+",\"grade\":"+call(rel,"getGrade")+",\"source_ratio\":"+call(rel,"getSourceRatio")+",\"target_ratio\":"+call(rel,"getTargetRatio")+"}");}return "["+String.join(",",rows)+"]";
 }
 static String controls(Object sys,Object factory,Object observed)throws Exception {
 Object sig=call(sys,"getSig");List<String> rows=new ArrayList<>();var create=c("sig.inter.AlterInter").getMethod("create",c("glyph.Glyph"),c("glyph.Shape"),Double.class,c("sheet.Staff"));var hf=c("sheet.symbol.InterFactory").getDeclaredField("systemHeads");hf.setAccessible(true);Object heads=hf.get(factory);double threshold=c("glyph.Grades").getField("minContextualGrade").getDouble(null);
 for(String mode:new String[]{"automatic","manual","frozen"})for(double grade:new double[]{((Number)call(observed,"getGrade")).doubleValue(),.05,.49,.5}) {
 Object alter=create.invoke(null,call(observed,"getGlyph"),call(observed,"getShape"),grade,call(observed,"getStaff"));call(alter,"setManual",mode.equals("manual"));if(mode.equals("frozen"))call(alter,"freeze");call(sig,"addVertex",alter);call(alter,"setLinks",heads);double cg=((Number)call(sig,"computeContextualGrade",alter)).doubleValue();
 rows.add("{\"mode\":\""+mode+"\",\"grade\":"+grade+",\"contextual_grade\":"+cg+",\"weak\":"+(cg<threshold&&!mode.equals("frozen"))+",\"manual\":"+call(alter,"isManual")+",\"frozen\":"+call(alter,"isFrozen")+",\"abnormal\":"+call(alter,"isAbnormal")+",\"supports\":"+supports(sig,alter)+"}");call(alter,"remove");}
 return "["+String.join(",",rows)+"]";
 }
 public void process(){try {
 Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");Map<Object,List<?>> optionals=new LinkedHashMap<>();call(make("sheet.symbol.SymbolsFilter",sheet),"process",optionals);List<String> systems=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){
 Object sig=call(sys,"getSig"),factory=make("sheet.symbol.InterFactory",sys);call(make("sheet.symbol.SymbolsBuilder",sys,factory),"buildSymbols",optionals);call(make("sheet.note.ChordsBuilder",sys),"buildRestChords");call(factory,"lateChecks");List<Object> alters=new ArrayList<>((List<?>)call(sig,"inters",c("sig.inter.AlterInter")));List<String> trials=new ArrayList<>(),before=new ArrayList<>();
 for(Object alter:alters){if(!alter.getClass().getSimpleName().equals("AlterInter"))continue;trials.add("{\"glyph\":"+glyph(call(alter,"getGlyph"))+",\"shape\":\""+call(alter,"getShape")+"\",\"staff\":"+call(call(alter,"getStaff"),"getId")+",\"controls\":"+controls(sys,factory,alter)+"}");before.add("{\"bounds\":"+rect(call(alter,"getBounds"))+",\"grade\":"+call(alter,"getGrade")+",\"contextual_grade\":"+call(sig,"computeContextualGrade",alter)+",\"supports\":"+supports(sig,alter)+"}");}
 call(sig,"contextualize");call(sig,"deleteWeakInters");List<String> after=new ArrayList<>();for(Object alter:alters)if(alter.getClass().getSimpleName().equals("AlterInter"))after.add("{\"bounds\":"+rect(call(alter,"getBounds"))+",\"removed\":"+call(alter,"isRemoved")+",\"contextual_grade\":"+call(alter,"getContextualGrade")+",\"manual\":"+call(alter,"isManual")+",\"abnormal\":"+call(alter,"isAbnormal")+"}");
 systems.add("{\"system\":"+call(sys,"getId")+",\"trials\":["+String.join(",",trials)+"],\"before_contextualize_and_purge\":["+String.join(",",before)+"],\"after_contextualize_and_purge\":["+String.join(",",after)+"]}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged image ordinary SymbolsStep buildSymbols then RestChords then lateChecks; actual Alter support contextualization and controlled own grades/manual/frozen; separate SIG contextualize/deleteWeakInters invocation, not full reduceLinks, no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
