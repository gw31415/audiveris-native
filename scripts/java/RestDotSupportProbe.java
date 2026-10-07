// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class RestDotSupportProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> rows=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){Object sig=call(sys,"getSig");
   for(Object rest:new ArrayList<>((Collection<?>)call(sig,"vertexSet"))){if(!rest.getClass().getSimpleName().equals("RestInter"))continue;
    for(Object rel:(Collection<?>)call(sig,"edgesOf",rest)){if(!rel.getClass().getSimpleName().equals("AugmentationRelation"))continue;Object dot=call(sig,"getEdgeSource",rel);
     List<Object> grades=new ArrayList<>();double old=((Number)call(rest,"getGrade")).doubleValue();
     for(double declared:new double[]{old,.49,.5}){call(rest,"setGrade",declared);grades.add(obj("declared_target_grade",declared,"original_contextual_target_grade",call(sig,"computeContextualGrade",rest)));}call(rest,"setGrade",old);
     rows.add(obj("rest_bounds",rect(call(rest,"getBounds")),"dot_bounds",rect(call(dot,"getBounds")),"source_ratio",call(rel,"getSourceRatio"),"target_ratio",call(rel,"getTargetRatio"),"declared_target_grade_controls",grades,"dot_part_matches_rest",call(dot,"getPart")==call(rest,"getPart"),"dot_voice_matches_rest",call(dot,"getVoice")==call(rest,"getVoice")));
    }
   }
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","saved original actual rest/dot ownership and AugmentationRelation target ratios; temporary declared target grade controls; not new full-image recognition", "cases",rows));System.exit(0);
 }
}
