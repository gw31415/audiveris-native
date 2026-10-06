// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class AugmentationGapProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();
  for(double dx:new double[]{Math.nextDown(2.0),2.0}) {
   Object rel=make("sig.relation.AugmentationRelation");call(rel,"setOutGaps",dx,0.0,1);
   double grade=((Number)call(rel,"getGrade")).doubleValue();double min=((Number)call(rel,"getMinGrade")).doubleValue();
   cases.add(obj("dx",dx,"dy",0.0,"grade",grade,"accepted",grade>=min));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared original AugmentationRelation zero-weight x-impact exhaustion; distinct from full image recognition","cases",cases));
 }
}
