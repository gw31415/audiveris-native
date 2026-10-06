// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamHeadCleanerProbe extends FoundationOverlapProbe {
    static Object cleanupState(Object sig,Object h,Object b)throws Exception {
        return obj("head_grade",call(h,"getGrade"),"head_contextual_grade",call(h,"getContextualGrade"),
            "beam_grade",call(b,"getGrade"),"beam_contextual_grade",call(b,"getContextualGrade"),
            "head_removed",call(h,"isRemoved"),"beam_removed",call(b,"isRemoved"),
            "live_vertices",((Collection<?>)call(sig,"vertexSet")).size(),
            "live_relations",((Collection<?>)call(sig,"edgeSet")).size());
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
        for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
        Object h=head(sys,physical(300,250,12,8,"VERTICAL",null));call(h,"setGrade",.4);
        Object b=beam(new double[]{290,210,350,210},4.,physical(290,208,60,4,"HORIZONTAL",null),false);
        call(sig,"addVertex",h);call(sig,"addVertex",b);
        call(sig,"addEdge",b,h,make("sig.relation.BeamHeadRelation",.7,true));
        call(sig,"computeContextualGrade",h);call(sig,"computeContextualGrade",b);
        Object before=cleanupState(sig,h,b),cleaner=make("sig.BeamHeadCleaner",sys);
        call(cleaner,"process");Object after=cleanupState(sig,h,b);call(cleaner,"process");Object repeated=cleanupState(sig,h,b);
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "scope","one actual original BeamHeadCleaner lifecycle; explicit declared SIG, not full-image post-LINKS SIG parity",
            "head_intrinsic",.4,"beam_intrinsic",.8,"relation_grade",.7,"on_beam_side",true,
            "before",before,"after",after,"repeat_is_idempotent",new ObjectMapper().valueToTree(after).equals(new ObjectMapper().valueToTree(repeated))));
        System.exit(0);
    }
}
