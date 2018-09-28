using System;
using AutoMapper;

namespace am {

    class Foo {
        public int Data { get; set; }
    }

    class FooDto {
        public int Data { get; set; }
    }

    class Program {
        static void Main (string[] args) {

            Mapper.Initialize (cfg => {
                cfg.CreateMap<Foo, FooDto> ();
            });

            var foo = new Foo {
                Data = 5
            };

            var fooDto = Mapper.Map<FooDto> (foo);

            Console.WriteLine (fooDto.Data);
        }
    }
}