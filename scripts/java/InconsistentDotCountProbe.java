// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class InconsistentDotCountProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  Object system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
  List<Object> heads=new ArrayList<>();
  for(Object stack:(List<?>)call(system,"getStacks"))for(Object chord:(Collection<?>)call(call(stack,"getFirstMeasure"),"getStandardChords"))if(chord.getClass().getSimpleName().equals("HeadChordInter"))heads.add(chord);
  heads.sort(Comparator.comparingInt(n->{try{return ((java.awt.Rectangle)call(n,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
  Object glyph=null;
  for(Object g:(Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities"))if(call(g,"getBounds").equals(new java.awt.Rectangle(303,235,5,4)))glyph=g;
  if(glyph==null)throw new IllegalStateException("original recognized glyph missing");
  List<Object> results=new ArrayList<>();
  for(boolean manual:new boolean[]{false,true}) {
   Object dot=make("sig.inter.AugmentationDotInter",glyph,.8);call(dot,"setManual",manual);
   Object link=call(dot,"lookupHeadLink",heads,system,call(system,"getProfile"));
   if(link==null)throw new IllegalStateException("original head link missing");
   Object note=link.getClass().getField("partner").get(link),chord=call(note,"getChord");
   call(dot,"setStaff",call(note,"getStaff"));call(sig,"addVertex",dot);call(link,"applyTo",dot);
   List<Object> before=new ArrayList<>();for(Object n:(Collection<?>)call(chord,"getNotes"))before.add(call(n,"getDotCount"));
   call(chord,"countDots");
   results.add(obj("declared_intrinsic_grade",.8,"manual",manual,"glyph_bounds",rect(call(glyph,"getBounds")),"chord_bounds",rect(call(chord,"getBounds")),"linked_note_bounds",rect(call(note,"getBounds")),"before_member_dot_counts",before,"rounded_chord_dots",call(chord,"getDotsNumber"),"dot_removed",call(dot,"isRemoved"),"duration",fraction(call(chord,"getDurationSansTuplet"))));
   if(!(Boolean)call(dot,"isRemoved"))call(dot,"remove");
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","separate declared-grade temporary augmentation on actual original saved glyph and recognized chord; original head lookup/countDots automatic-vs-manual controls, no image result replacement","controls",results));System.exit(0);
 }
}
