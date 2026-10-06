// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class AugmentationStateProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  List<Object> rows=new ArrayList<>(),dots=new ArrayList<>(),pauseControls=new ArrayList<>();
  for(Object system:(List<?>)call(sheet,"getSystems")) {
   Object sig=call(system,"getSig");
   List<Object> heads=new ArrayList<>(),rests=new ArrayList<>();
   for(Object stack:(List<?>)call(system,"getStacks"))for(Object chord:(Collection<?>)call(call(stack,"getFirstMeasure"),"getStandardChords")) {
    if(chord.getClass().getSimpleName().equals("HeadChordInter"))heads.add(chord);
    List<Object> notes=new ArrayList<>();
    for(Object note:(Collection<?>)call(chord,"getNotes")) {
     if(note.getClass().getSimpleName().equals("RestInter"))rests.add(note);
     Object dot=call(note,"getFirstAugmentationDot");
     notes.add(obj("bounds",rect(call(note,"getBounds")),"dot_count",call(note,"getDotCount"),"first_dot_bounds",dot==null?null:rect(call(dot,"getBounds"))));
    }
    rows.add(obj("bounds",rect(call(chord,"getBounds")),"bounds_with_dots",rect(call(chord,"getBoundsWithDots")),"dots_number",call(chord,"getDotsNumber"),"intrinsic_without_dots",fraction(call(chord,"getDurationSansDotOrTuplet")),"intrinsic_with_dots",fraction(call(chord,"getDurationSansTuplet")),"notes",notes));
   }
   heads.sort(Comparator.comparingInt(h->{try{return ((java.awt.Rectangle)call(h,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
   for(Object dot:(Collection<?>)call(sig,"vertexSet")) {
    if(!dot.getClass().getSimpleName().equals("AugmentationDotInter"))continue;
    List<Object> links=new ArrayList<>();
    for(Object rel:(Collection<?>)call(sig,"edgesOf",dot)) {
     String kind=rel.getClass().getSimpleName();
     if(!kind.equals("AugmentationRelation")&&!kind.equals("DoubleDotRelation"))continue;
     Object source=call(sig,"getEdgeSource",rel),target=call(sig,"getEdgeTarget",rel);
     links.add(obj("kind",kind,"source_bounds",rect(call(source,"getBounds")),"target_bounds",rect(call(target,"getBounds")),"grade",call(rel,"getGrade"),"dx",call(rel,"getDx"),"dy",call(rel,"getDy")));
    }
    Object headLink=call(dot,"lookupHeadLink",heads,system,call(system,"getProfile"));
    Object partner=headLink==null?null:headLink.getClass().getField("partner").get(headLink);
    if(partner!=null){
     java.awt.Point dotCenter=(java.awt.Point)call(dot,"getCenter"),ref=(java.awt.Point)call(partner,"getCenterRight");
     Object precise=make("sig.relation.AugmentationRelation");
     double il=((Number)call(call(sheet,"getScale"),"getInterline")).doubleValue();
     call(precise,"setOutGaps",(dotCenter.x-ref.x)/il,Math.abs(dotCenter.y-ref.y)/il,call(system,"getProfile"));
     links.add(obj("kind","original_geometry_recomputed_relation","grade",call(precise,"getGrade"),"dx",call(precise,"getDx"),"dy",call(precise,"getDy")));
    }
    double declaredScore=0.18859216955421593;
    Object pause=c("sig.inter.AbstractPauseInter").getMethod("createValidAdded",c("glyph.Glyph"),c("glyph.Shape"),Double.class,c("sheet.Staff"),List.class).invoke(null,call(dot,"getGlyph"),c("glyph.Shape").getField("CAESURA").get(null),0.8*declaredScore,call(dot,"getStaff"),heads);
    if(pause!=null){
     Object rel=((Collection<?>)call(sig,"edgesOf",pause)).iterator().next();
     Object ch=call(sig,"getEdgeSource",rel);
     Object contextual=call(sig,"computeContextualGrade",pause);
     pauseControls.add(obj("declared_classifier_score",declaredScore,"bounds",rect(call(pause,"getBounds")),"partner_bounds",rect(call(ch,"getBounds")),"partner_grade",call(ch,"getGrade"),"relation_grade",call(rel,"getGrade"),"dx",call(rel,"getDx"),"grade",call(pause,"getGrade"),"contextual_grade",contextual,"weak",((Number)contextual).doubleValue()<0.5));
     call(pause,"remove");
    }
    dots.add(obj("bounds",rect(call(dot,"getBounds")),"grade",call(dot,"getGrade"),"best_grade",call(dot,"getBestGrade"),"profile",call(dot,"getProfile"),"system_profile",call(system,"getProfile"),"links",links,"lookup_head_bounds",partner==null?null:rect(call(partner,"getBounds"))));
   }
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","saved recognized dot ownership/duration getters and exact original geometry regrading; separate declared CAESURA classifier score uses original factory/context on temporary original Inter; no declared clocks/music changes","chords",rows,"dots",dots,"declared_pause_controls",pauseControls));System.exit(0);
 }
}
