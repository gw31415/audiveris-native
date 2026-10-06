// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class InjectionControlsProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();int[][][] matrices={{{60,60}},{{1,1,60,60},{1,1,60,60}},{{1,2,60,60},{1,20,60,60}}};String[] names={"no-link-cost-tie-first-range","two-incomings-equal-cost-stable-injection","global-cost-beats-greedy"};
  Class<?> distance=c("math.InjectionSolver$Distance");
  for(int i=0;i<matrices.length;i++){int[][] table=matrices[i];Object callback=Proxy.newProxyInstance(distance.getClassLoader(),new Class<?>[]{distance},(proxy,method,values)->table[(Integer)values[0]][(Integer)values[1]]);Object solver=make("math.InjectionSolver",table.length,table[0].length,callback),cost=make("util.WrappedInteger",(Object)null);Object links=call(solver,"solve",cost);cases.add(obj("name",names[i],"costs",table,"links",links,"cost",cost.getClass().getField("value").get(cost)));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("scope","three original InjectionSolver method controls, not native image polyphony","cases",cases));System.exit(0);
 }
}
