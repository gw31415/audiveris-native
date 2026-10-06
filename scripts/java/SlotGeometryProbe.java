// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.awt.geom.Point2D;
import com.fasterxml.jackson.databind.ObjectMapper;
public class SlotGeometryProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> measures=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks")) {
   List<Object> slots=new ArrayList<>();
   for(Object slot:(List<?>)call(stack,"getSlots"))slots.add(obj("id",call(slot,"getId"),"time_offset",String.valueOf(call(slot,"getTimeOffset")),"x_offset",call(slot,"getXOffset")));
   List<Object> queries=new ArrayList<>();Object measure=call(stack,"getFirstMeasure"),staff=call(call(measure,"getPart"),"getFirstStaff");
   Point2D left=(Point2D)call(measure,"getSidePoint",c("util.HorizontalSide").getField("LEFT").get(null),staff);
   List<?> actual=(List<?>)call(stack,"getSlots");
   if(!actual.isEmpty()) {
    double first=((Number)call(actual.get(0),"getXOffset")).doubleValue(),second=((Number)call(actual.get(1),"getXOffset")).doubleValue();
    for(double x:new double[]{left.getX()+first,left.getX()+(first+second)/2.,left.getX()+((Number)call(actual.get(actual.size()-1),"getXOffset")).doubleValue()+5.}) {
     double y=((Number)call(((List<?>)call(staff,"getLines")).get(2),"yAt",x)).doubleValue();
     Object closest=call(stack,"getClosestSlot",new Point2D.Double(x,y));
     queries.add(obj("point",List.of(x,y),"slot_id",closest==null?null:call(closest,"getId")));
    }
   } else {double y=((Number)call(((List<?>)call(staff,"getLines")).get(2),"yAt",left.getX())).doubleValue();Object closest=call(stack,"getClosestSlot",new Point2D.Double(left.getX(),y));queries.add(obj("point",List.of(left.getX(),y),"slot_id",closest==null?null:call(closest,"getId")));}
   measures.add(obj("number",call(stack,"getIdValue"),"slots",slots,"queries",queries));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original saved stack slot computeXOffset/getXOffset on actual image chord centers; zero-skew current one-part/one-staff geometry","measures",measures));System.exit(0);
 }
}
