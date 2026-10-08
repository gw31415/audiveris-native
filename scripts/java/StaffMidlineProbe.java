// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.geom.Point2D;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Actual original Staff/StaffLine selection, interpolation and extrapolation. */
public class StaffMidlineProbe extends VoiceLifecycleProbe {
    static List<Object> samples(Object staff,int[] xs)throws Exception {
        Object line=call(staff,"getMidLine");List<Object> rows=new ArrayList<>();
        java.lang.reflect.Method yAt=line.getClass().getMethod("yAt",int.class);
        for(int x:xs)rows.add(obj("x",x,"y",yAt.invoke(line,x),"same_middle_line",call(staff,"getMidLine")==line));
        return rows;
    }
    public static void main(String[] args)throws Exception {
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),staff=call(system,"getFirstStaff");
        int[] xs={-20,0,145,146,288,299,360,459,470,630,981,1100,2000};
        List<Object> cases=new ArrayList<>();int[] counts={1,2,4,5,6},declaredX={-20,-5,0,8,13,24,35,45,80};
        for(int count:counts){List<Object> lines=new ArrayList<>(),declared=new ArrayList<>();for(int i=0;i<count;i++){
            List<Point2D> points=List.of(new Point2D.Double(-5,100.+i*11),new Point2D.Double(8,106.+i*11),new Point2D.Double(24,98.+i*11),new Point2D.Double(45,112.+i*11));
            lines.add(make("sheet.StaffLine",points,1.));declared.add(points.stream().map(p->List.of(p.getX(),p.getY())).toList());
        }Object control=make("sheet.Staff",1,-5.,45.,11,lines);List<Object> retained=new ArrayList<>();for(Object line:lines){List<Object> coords=new ArrayList<>();for(Object p:(List<?>)call(line,"getPoints")){Point2D pt=(Point2D)p;coords.add(List.of(pt.getX(),pt.getY()));}retained.add(coords);}cases.add(obj("line_count",count,"lines",declared,"retained_lines",retained,"middle_index",lines.indexOf(call(control,"getMidLine")),"samples",samples(control,declaredX)));}
        Object mid=call(staff,"getMidLine");List<Object> originalPoints=new ArrayList<>();for(Object p:(List<?>)call(mid,"getPoints")){Point2D pt=(Point2D)p;originalPoints.add(List.of(pt.getX(),pt.getY()));}
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual loaded source Staff middle-line samples and five independently declared real Staff/StaffLine controls; even lower-index selection, cubic interpolation/global-slope extrapolation, ties-even yAt(int); no native output input or archive creation","source",obj("staff_id",call(staff,"getId"),"middle_points",originalPoints,"samples",samples(staff,xs)),"cases",cases));
    }
}
