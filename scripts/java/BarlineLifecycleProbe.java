// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BarlineLifecycleProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),bar=null;
  for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("BarlineInter")&&call(n,"getBounds").equals(new java.awt.Rectangle(528,219,2,47)))bar=n;
  if(bar==null)throw new IllegalStateException("actual original GRID bar not found");
  Object staff=call(bar,"getStaff"),glyph=call(bar,"getGlyph");
  List<Object> before=new ArrayList<>(),after=new ArrayList<>();
  for(Object b:(Collection<?>)call(staff,"getBarlines"))before.add(obj("id",call(b,"getId"),"bounds",rect(call(b,"getBounds"))));
  Object ensembles=call(bar,"getAllEnsembles");
  call(bar,"remove");
  for(Object b:(Collection<?>)call(staff,"getBarlines"))after.add(obj("id",call(b,"getId"),"bounds",rect(call(b,"getBounds"))));
  boolean live=((Collection<?>)call(sig,"vertexSet")).contains(bar);
  call(bar,"remove");
  boolean physical=((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities")).contains(glyph);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual original GRID Inter from unchanged negative rest image; direct original BarlineInter.remove/Staff collection/SIG lifecycle; not positive dotted-rest recognition","bar",obj("kind",bar.getClass().getSimpleName(),"bounds",rect(call(bar,"getBounds")),"staff_id",call(staff,"getId"),"grade",call(bar,"getGrade"),"shape",call(bar,"getShape").toString(),"median",line(call(bar,"getMedian")),"width",call(bar,"getWidth"),"frozen",call(bar,"isFrozen"),"manual",call(bar,"isManual")),"before_staff_barlines",before,"after_staff_barlines",after,"before_ensemble_count",ensembles==null?0:((Collection<?>)ensembles).size(),"removed",call(bar,"isRemoved"),"live_in_SIG",live,"physical_glyph_retained_in_index",physical,"repeated_remove_is_idempotent",!((Collection<?>)call(sig,"vertexSet")).contains(bar)));
  System.exit(0);
 }
}
