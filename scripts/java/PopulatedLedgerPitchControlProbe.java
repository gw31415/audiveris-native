// SPDX-License-Identifier: AGPL-3.0-or-later
// Supplied staff and straight GeoPath ledger curves; unmodified Staff.pitchPositionOf.
import java.lang.reflect.*;
import java.awt.geom.Point2D;
import java.nio.file.*;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class PopulatedLedgerPitchControlProbe {
 public static void main(String[] args) throws Exception {
  Class<?> staffClass=Class.forName("org.audiveris.omr.sheet.Staff"),lineClass=Class.forName("org.audiveris.omr.sheet.grid.LineInfo"); List<Object> lines=new ArrayList<>();List<Object> geometry=new ArrayList<>();
  for(int n=0;n<5;n++) {
   final double base=100.+15.75*n,slope=.1+.025*n;
   lines.add(Proxy.newProxyInstance(lineClass.getClassLoader(),new Class[]{lineClass},(o,m,a)->{
    if(m.getName().equals("yAt")) {double y=base+slope*((Number)a[0]).doubleValue();if(m.getReturnType()==int.class)return (int)Math.rint(y);return y;}
    if(m.getName().equals("getThickness"))return 1.;
    throw new UnsupportedOperationException(m.getName());
   }));
   geometry.add(Map.of("points",List.of(new double[]{0,base},new double[]{128,base+128*slope}),"thickness",1.));
  }
  Object staff=staffClass.getConstructor().newInstance();
  for(var e:Map.of("lines",(Object)lines,"specificInterline",(Object)16).entrySet()){Field f=staffClass.getDeclaredField(e.getKey());f.setAccessible(true);f.set(staff,e.getValue());}
  Field ledger=staffClass.getDeclaredField("ledgerLineMap");ledger.setAccessible(true);if(!((Map<?,?>)ledger.get(staff)).isEmpty())throw new AssertionError("nonempty ledger map");
  Class<?> pathClass=Class.forName("org.audiveris.omr.math.GeoPath");
  @SuppressWarnings("unchecked") Map<Integer,Object> ledgerMap=(Map<Integer,Object>)ledger.get(staff);
  List<Object> configurations=new ArrayList<>();
  for(String configuration:List.of("consecutive","gap_after_first","gap_at_first","negative_only")) {
   ledgerMap.clear();List<Object> curves=new ArrayList<>();
   int[] indices=switch(configuration) {
    case "consecutive" -> new int[]{-2,-1,1,2};
    case "gap_after_first" -> new int[]{-3,-1,1,3};
    case "gap_at_first" -> new int[]{-2,2};
    default -> new int[]{-2,-1};
   };
   for(int index:indices) {
    double base=index<0?(index==-1?88:70):index==1?182:211;
    double slope=index<0?(index==-1?.11:.13):index==1?.18:.16;
    Object path=pathClass.getConstructor().newInstance();
    pathClass.getMethod("moveTo",double.class,double.class).invoke(path,0.,base);
    pathClass.getMethod("lineTo",double.class,double.class).invoke(path,128.,base+128*slope);
    ledgerMap.put(index,path);
    curves.add(Map.of("index",index,"points",List.of(new double[]{0,base},new double[]{128,base+128*slope})));
   }
   List<Object> cases=new ArrayList<>();
   for(double x:new double[]{-16,0,64,127,160}) {
    double top=100.+.1*x,bottom=163.+.2*x;
    for(double y:new double[]{top-60,top-30,top-12,top-.25,top,(top+bottom)/2,bottom,bottom+.25,bottom+19,bottom+40,bottom+80})cases.add(Map.of("x",x,"y",y,"pitch",staffClass.getMethod("pitchPositionOf",Point2D.class).invoke(staff,new Point2D.Double(x,y))));
   }
   configurations.add(Map.of("name",configuration,"ledgers",curves,"cases",cases));
  }
  new ObjectMapper().writeValue(Path.of(args[0]).toFile(),Map.of("native_outputs_used",false,"scope","Unmodified original Staff.pitchPositionOf; supplied sloped five-line staff and straight GeoPath ledgers. Consecutive/gapped signed indices, interpolation, absent-first-key fallback, external abscissas; not native ledger discovery or real-score positive conversion.","specific_interline",16,"lines",geometry,"configurations",configurations));
 }
}
