// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.geom.*;import java.lang.reflect.*;import java.nio.file.*;import java.util.*;
public class SlurNativeCurveLookupProbe extends SlurOwnedLookupProbeV2 {
 public static void main(String[]args)throws Exception{
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");Object linker=c("sheet.curve.SlurLinker").getConstructor(c("sheet.Sheet")).newInstance(sheet);Object manager=call(sheet,"getSystemManager");List<Object>rows=new ArrayList<>(),unrouted=new ArrayList<>();
  for(String line:Files.readAllLines(Path.of(args[1]))){String[]f=line.split("\\t");String[]values=f[3].split(",");double[]p=new double[8];for(int i=0;i<8;i++)p[i]=Double.parseDouble(values[i]);boolean above=Boolean.parseBoolean(f[1]);Object slur=c("sig.inter.SlurInter").getConstructor(boolean.class,Double.class).newInstance(above,Double.valueOf(f[2]));set(slur,"above",above);set(slur,"curve",new CubicCurve2D.Double(p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7]));
   @SuppressWarnings("unchecked") Map<Object,Area>areas=(Map<Object,Area>)call(linker,"defineAreaPair",slur);
   @SuppressWarnings("unchecked") List<Object>systems=new ArrayList<>((List<Object>)call(manager,"getSystemsOf",areas.get(sides(0)).getBounds(),null));
   systems.retainAll((List<?>)call(manager,"getSystemsOf",areas.get(sides(1)).getBounds(),null));
   if(systems.isEmpty())unrouted.add(map("native_case",Integer.parseInt(f[0]),"curve",bezier(call(slur,"getCurve"))));
   for(Object sys:systems){List<Object>chords=new ArrayList<>();for(Object inter:(Set<?>)call(call(sys,"getSig"),"vertexSet"))if(c("sig.inter.HeadChordInter").isInstance(inter))chords.add(inter);
    @SuppressWarnings("unchecked") Map<String,Object>row=(Map<String,Object>)observe("native_curve_"+f[0]+"_system_"+id(sys),"native_generated_full_precision_curve_against_original_real_owners",slur,linker,sys,chords);row.put("native_case",Integer.parseInt(f[0]));rows.add(row);
   }
  }
  Files.writeString(Path.of(args[2]),json(map("scope","Native Rust-generated full-precision cubic candidates are the inputs. Source SystemManager selects relevant Systems from both area bounds, then original SlurLinker uses every actual original system HeadChord. This is a geometric transfer diagnostic against original owners, NOT actual native-owner integration, source curve extraction, Slur acceptance, callbacks, cleanup or quality parity. Raw-support failures remain in input provenance and cannot be admitted by this lookup.","interline",call(call(sheet,"getScale"),"getInterline"),"unrouted",unrouted,"rows",rows))+"\n");
 }
}
