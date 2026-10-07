// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class RestDotOrphanProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> rows=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){Object sig=call(sys,"getSig");
   for(Object rest:new ArrayList<>((Collection<?>)call(sig,"vertexSet"))){if(!rest.getClass().getSimpleName().equals("RestInter"))continue;
    for(Object rel:new ArrayList<>((Collection<?>)call(sig,"edgesOf",rest))){if(!rel.getClass().getSimpleName().equals("AugmentationRelation"))continue;
     Object dot=call(sig,"getEdgeSource",rel),glyph=call(dot,"getGlyph"),chord=call(rest,"getChord"),measure=call(chord,"getMeasure"),reducer=make("sig.SigReducer",sys,true);
     boolean beforeAbnormal=(Boolean)call(dot,"isAbnormal"),beforeMember=((Collection<?>)call(measure,"getAugmentationDots")).contains(dot);
     call(rest,"setGrade",.49);double targetContext=((Number)call(sig,"computeContextualGrade",rest)).doubleValue();
     invoke(reducer,"contextualizeAndPurge");
     Object afterPurge=obj("rest_removed",call(rest,"isRemoved"),"rest_chord_removed",call(chord,"isRemoved"),"dot_removed",call(dot,"isRemoved"),"dot_abnormal",call(dot,"isAbnormal"),"dot_best_grade",call(dot,"getBestGrade"),"augmentation_edge_present",((Collection<?>)call(sig,"edgesOf",dot)).contains(rel));
     Object modifications=invoke(reducer,"checkAugmentationDots");
     Object afterCheck=obj("dot_removed",call(dot,"isRemoved"),"live_in_SIG",((Collection<?>)call(sig,"vertexSet")).contains(dot),"measure_dot_present",((Collection<?>)call(measure,"getAugmentationDots")).contains(dot),"physical_glyph_retained",((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities")).contains(glyph),"repeat_check_modifications",invoke(reducer,"checkAugmentationDots"));
     rows.add(obj("rest_bounds",rect(call(rest,"getBounds")),"dot_bounds",rect(call(dot,"getBounds")),"before_dot_abnormal",beforeAbnormal,"before_measure_dot_present",beforeMember,"declared_target_grade",.49,"target_contextual_grade",targetContext,"after_original_contextualizeAndPurge",afterPurge,"original_checkAugmentationDots_modifications",modifications,"after_original_check",afterCheck));
    }
   }
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual saved successful rest/dot Inter; temporary target grade .49; original contextualizeAndPurge and checkAugmentationDots lifecycle; not automatic image weak-rest evidence","cases",rows));System.exit(0);
 }
}
