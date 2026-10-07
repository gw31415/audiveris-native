// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotChainDetachProbe extends RookieMappingProbe {
 static Object state(Object sig,Object chord,Object measure,Object first,Object second,Object head)throws Exception {
  return obj("first_removed",call(first,"isRemoved"),"second_removed",call(second,"isRemoved"),"first_abnormal",call(first,"isAbnormal"),"second_abnormal",call(second,"isAbnormal"),"first_measure_member",((Collection<?>)call(measure,"getAugmentationDots")).contains(first),"second_measure_member",((Collection<?>)call(measure,"getAugmentationDots")).contains(second),"head_removed",call(head,"isRemoved"),"chord_removed",call(chord,"isRemoved"),"chord_dots",call(chord,"getDotsNumber"),"chord_intrinsic_duration",fraction(call(chord,"getDurationSansTuplet")));
 }
 public static void main(String[] args)throws Exception {
  List<Object> rows=new ArrayList<>();
  for(boolean warm:new boolean[]{false,true}){
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object sig=call(sys,"getSig"),dd=null;for(Object rel:(Collection<?>)call(sig,"edgeSet"))if(rel.getClass().getSimpleName().equals("DoubleDotRelation")){dd=rel;break;}if(dd==null)continue;
   Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd),aug=null;
   for(Object rel:(Collection<?>)call(sig,"edgesOf",first))if(rel.getClass().getSimpleName().equals("AugmentationRelation")){aug=rel;break;}
   Object head=call(sig,"getEdgeTarget",aug),chord=call(head,"getChord"),measure=call(chord,"getMeasure");if(((Collection<?>)call(chord,"getNotes")).size()!=1)throw new IllegalStateException("actual chord not single-note");
   if(warm){call(first,"getPart");call(second,"getPart");}
   Object before=state(sig,chord,measure,first,second,head);call(sig,"removeEdge",aug);Object afterEdge=state(sig,chord,measure,first,second,head);
   Object reducer=make("sig.SigReducer",sys,true);Object mods=invoke(reducer,"checkAugmentationDots"),after=state(sig,chord,measure,first,second,head);boolean physical=true;
   for(Object d:List.of(first,second))physical&=((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities")).contains(call(d,"getGlyph"));
   rows.add(obj("part_getters_warmed_before_detachment",warm,"first_bounds",rect(call(first,"getBounds")),"second_bounds",rect(call(second,"getBounds")),"head_bounds",rect(call(head,"getBounds")),"chord_bounds",rect(call(chord,"getBounds")),"before",before,"after_original_augmentation_edge_remove_callback",afterEdge,"original_target_check_modifications",mods,"after_original_target_check",after,"repeat_target_check_modifications",invoke(reducer,"checkAugmentationDots"),"physical_glyphs_retained",physical));
  }
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","actual saved successful double-dot chain; explicit original augmentation edge detachment/callback and target check; no new automatic image/export success and no declared clocks","cases",rows));System.exit(0);
 }
}
