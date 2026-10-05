// SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit numerical inputs, original Population/Gaussian/Clustering.EM only.
import java.util.*;import java.nio.file.*;import java.lang.reflect.*;
public class ClusteringControlProbe {
 static String number(double d){return Double.isFinite(d)?Double.toString(d):"\""+d+"\"";}
 static String array(double[] a){List<String> s=new ArrayList<>();for(double v:a)s.add(number(v));return "["+String.join(",",s)+"]";}
 public static void main(String[] args)throws Exception{
  Class<?> gc=Class.forName("org.audiveris.omr.math.Clustering$Gaussian"),law=Class.forName("org.audiveris.omr.math.Clustering$Law"),cl=Class.forName("org.audiveris.omr.math.Clustering"),pop=Class.forName("org.audiveris.omr.math.Population");
  String[] names={"empty","singleton","constant","half-even-low","half-even-high","half-even-negative","unequal-degenerate","overlapping","identical-populations","weighted-overlap","close-laws","separated","three-overlap","underflow"};
  double[][][] inputs={ {},{{12}},{{7,7,7}},{{10,11}},{{11,12}},{{-1,0}},{{0,0,0,0},{10}},{{0,1,2},{1,2,3}},{{0,1},{0,1}},{{0,0,1,1,1,2},{1,2}},{{1,1+1e-11},{1+2e-11,1+3e-11}},{{0,0,0},{100,100}},{{0,1,2},{1,2,3},{2,3,4}},{{-1000,1000},{-2000,2000}} };
  List<String> records=new ArrayList<>();for(int c=0;c<names.length;c++){
   List<Double> vals=new ArrayList<>();List<String> populations=new ArrayList<>(),initials=new ArrayList<>();Object laws=Array.newInstance(law,inputs[c].length);
   for(int i=0;i<inputs[c].length;i++){Object p=pop.getConstructor().newInstance();for(double v:inputs[c][i]){GridProbe.call(p,"includeValue",v);vals.add(v);}double mean=(Double)GridProbe.call(p,"getMeanValue");Array.set(laws,i,gc.getConstructor(double.class,double.class).newInstance(mean,1.0));populations.add(array(inputs[c][i]));initials.add(number(mean));}
   double[] x=new double[vals.size()];for(int i=0;i<x.length;i++)x[i]=vals.get(i);double[] pi=(double[])cl.getMethod("EM",double[].class,laws.getClass()).invoke(null,x,laws);List<String> finalLaws=new ArrayList<>();for(int i=0;i<pi.length;i++){Object g=Array.get(laws,i);double mean=(Double)GridProbe.get(g,"mean"),sigma=(Double)GridProbe.get(g,"sigma");finalLaws.add("{\"mean\":"+number(mean)+",\"sigma\":"+number(sigma)+",\"pi\":"+number(pi[i])+",\"mean_raw_bits_hex\":\""+Long.toHexString(Double.doubleToRawLongBits(mean))+"\",\"rounded_offset\":"+(int)Math.rint(mean)+"}");}
   records.add("{\"name\":\""+names[c]+"\",\"populations\":["+String.join(",",populations)+"],\"flattened_values\":"+array(x)+",\"initial_means\":["+String.join(",",initials)+"],\"initial_sigma\":1.0,\"final_laws\":["+String.join(",",finalLaws)+"]}");
  }
  List<String> proba=new ArrayList<>();for(double sigma:new double[]{0,Math.nextDown(1e-10),1e-10,Math.nextUp(1e-10)})for(double x:new double[]{0,Math.nextDown(1e-10),1e-10,Math.nextUp(1e-10),-1e-10}){Object g=gc.getConstructor(double.class,double.class).newInstance(0.0,sigma);double value=(Double)GridProbe.call(g,"proba",x);proba.add("{\"mean\":0.0,\"sigma\":"+number(sigma)+",\"x\":"+number(x)+",\"proba\":"+number(value)+"}");}
  Files.writeString(Path.of(args[0]),"{\"controlled_input\":true,\"original_methods\":[\"Population.getMeanValue\",\"Clustering.EM\",\"Gaussian.proba\"],\"nonfinite_encoding\":\"Double.toString strings, never fabricated finite numbers\",\"cases\":["+String.join(",",records)+"],\"gaussian_probability_controls\":["+String.join(",",proba)+"]}\n");System.exit(0);
 }
}
