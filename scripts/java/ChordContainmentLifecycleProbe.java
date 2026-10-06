// SPDX-License-Identifier: AGPL-3.0-or-later
// Original typed chord cache invalidation on actual member/ChordStem edges.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordContainmentLifecycleProbe extends FoundationOverlapProbe {
    static Object snapshot(Object chord,Map<Object,String> labels) throws Exception {
        List<String> ms=new ArrayList<>();for(Object h:(Collection<?>)call(chord,"getMembers"))ms.add(labels.get(h));
        Object leading=call(chord,"getLeadingNote");
        return obj("members",ms,"grade",call(chord,"getGrade"),"contextual_grade",call(chord,"getContextualGrade"),"bounds",call(chord,"getBounds")==null?null:rect(call(chord,"getBounds")),"leading",labels.get(leading),"head_location",leading==null?null:point(call(chord,"getHeadLocation")),"tail_location",leading==null?null:point(call(chord,"getTailLocation")));
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
        for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
        Map<Object,String> labels=new IdentityHashMap<>();List<Object> nodes=new ArrayList<>(),states=new ArrayList<>();
        Object a=head(sys,physical(88,120,12,8,"VERTICAL",null)),b=head(sys,physical(88,140,12,8,"VERTICAL",null));
        call(a,"setGrade",.4);call(b,"setGrade",.8);call(a,"setContextualGrade",.6);call(b,"setContextualGrade",.9);
        Object stem=make("sig.inter.StemInter",physical(100,80,2,60,"VERTICAL",null),.8),chord=make("sig.inter.HeadChordInter",.8);
        labels.put(a,"head0");labels.put(b,"head1");labels.put(stem,"stem");labels.put(chord,"chord");
        for(Object n:List.of(a,b,stem,chord))call(sig,"addVertex",n);
        call(chord,"setStaff",call(sys,"getFirstStaff"));
        for(Object n:List.of(a,b,stem))nodes.add(obj("label",labels.get(n),"kind",n.getClass().getSimpleName(),"shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"grade",call(n,"getGrade"),"contextual_grade",call(n,"getContextualGrade"),"glyph",glyph(call(n,"getGlyph"))));
        call(chord,"addMember",a);states.add(obj("operation","add-first-member","state",snapshot(chord,labels)));
        call(chord,"addMember",b);states.add(obj("operation","add-second-member","state",snapshot(chord,labels)));
        call(chord,"setStem",stem);states.add(obj("operation","add-stem","state",snapshot(chord,labels)));
        call(chord,"removeMember",b);states.add(obj("operation","remove-second-member","state",snapshot(chord,labels)));
        call(sig,"removeEdge",chord,stem);states.add(obj("operation","remove-stem","state",snapshot(chord,labels)));
        call(chord,"removeMember",a);states.add(obj("operation","remove-last-member","state",snapshot(chord,labels)));
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","one actual original chord lifecycle, six ordered member/stem relation operations; no vertex deletion, full Part/lyrics cleanup or raster creation","nodes",nodes,"states",states));System.exit(0);
    }
}
