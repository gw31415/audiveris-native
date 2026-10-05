// SPDX-License-Identifier: AGPL-3.0-or-later
// Supplied staff curves; calls unmodified original Staff.pitchPositionOf.
import java.lang.reflect.*;
import java.awt.geom.Point2D;
import java.nio.file.*;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class EmptyLedgerPitchControlProbe {
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
  List<Object> cases=new ArrayList<>();
  for(double x:new double[]{0,64,127}) {
   double top=100.+.1*x,bottom=163.+.2*x;
   for(double y:new double[]{top-24,top-.25,top,(top+bottom)/2,bottom,bottom+.25,bottom+24})cases.add(Map.of("x",x,"y",y,"pitch",staffClass.getMethod("pitchPositionOf",Point2D.class).invoke(staff,new Point2D.Double(x,y))));
  }
  new ObjectMapper().writeValue(Path.of(args[0]).toFile(),Map.of("native_outputs_used",false,"scope","Unmodified original Staff.pitchPositionOf; supplied five sloped lines, measured specificInterline=16, empty original ledgerLineMap; no staff association or positive raster conversion claimed","specific_interline",16,"lines",geometry,"cases",cases));
 }
}
