// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.geom.Point2D;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamStaffNeighborhoodProbe extends VoiceLifecycleProbe {
    static Object rect(Object value) { java.awt.Rectangle r=(java.awt.Rectangle)value;return List.of(r.x,r.y,r.width,r.height); }
    public static void main(String[] args)throws Exception {
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> rows=new ArrayList<>();
        for(Object system:(List<?>)call(sheet,"getSystems")) {
            List<?> staffs=(List<?>)call(system,"getStaves");List<Object> points=new ArrayList<>(),parts=new ArrayList<>();
            for(Object part:(List<?>)call(system,"getParts")){List<Object> ids=new ArrayList<>();for(Object staff:(List<?>)call(part,"getStaves"))ids.add(call(staff,"getId"));parts.add(obj("id",call(part,"getId"),"staff_ids",ids,"bounds",rect(call(part,"getAreaBounds"))));}
            for(Object staff:staffs) {
                double x=(((Number)call(staff,"getAbscissa",c("util.HorizontalSide").getField("LEFT").get(null))).doubleValue()+((Number)call(staff,"getAbscissa",c("util.HorizontalSide").getField("RIGHT").get(null))).doubleValue())/2.;
                double top=((Number)call(call(staff,"getFirstLine"),"yAt",x)).doubleValue(),bottom=((Number)call(call(staff,"getLastLine"),"yAt",x)).doubleValue();
                for(double y:new double[]{top-.5,top,(top+bottom)/2.,bottom,bottom+.5})points.add(obj("kind","staff_core_boundary","point",List.of(x,y)));
                points.add(obj("kind","outside_left","point",List.of(-1.,(top+bottom)/2.)));points.add(obj("kind","outside_right","point",List.of(((Number)call(sheet,"getWidth")).doubleValue(),(top+bottom)/2.)));
            }
            for(Object beam:(Collection<?>)call(call(system,"getSig"),"vertexSet"))if(c("sig.inter.AbstractBeamInter").isInstance(beam)){java.awt.Point center=(java.awt.Point)call(beam,"getCenter");points.add(obj("kind","actual_beam_center","point",List.of((double)center.x,(double)center.y)));}
            List<Object> cases=new ArrayList<>();for(Object value:points){Map<?,?> row=(Map<?,?>)value;List<?> point=(List<?>)row.get("point");Point2D p=new Point2D.Double(((Number)point.get(0)).doubleValue(),((Number)point.get(1)).doubleValue());List<Object> selected=new ArrayList<>(),selectedParts=new ArrayList<>(),bounds=new ArrayList<>();for(Object staff:(List<?>)call(system,"getStavesAround",p)){selected.add(call(staff,"getId"));Object part=call(staff,"getPart");selectedParts.add(call(part,"getId"));bounds.add(rect(call(part,"getAreaBounds")));}cases.add(obj("kind",row.get("kind"),"point",point,"staff_ids",selected,"part_ids",selectedParts,"part_bounds",bounds));}
            rows.add(obj("system",call(system,"getId"),"parts",parts,"cases",cases));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual original SystemInfo.getStavesAround and Staff->Part area bounds on complete real-image original book; core boundaries/outside-page/actual beam centers, no native outputs or modified source","systems",rows));
    }
}
