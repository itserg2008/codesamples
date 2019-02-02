using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace toppings
{    
    class Program
    {        
        public const string ORDERS_JSON_URL = "http://localhost:5000/pizzas.json";
        
        public const int MAX_RECORDS_DISPLAYED = 20;
        
        class PizzaOrder{
            public List<string> toppings;
        };
        
        public static async Task MainAsync(string[] args)
        {
            using(var client = new HttpClient())
            {
                try
                {
                    var response = await client.GetAsync(ORDERS_JSON_URL);
                    var stringResult = await response.Content.ReadAsStringAsync();
                    var pizzaOrders = JsonConvert.DeserializeObject<PizzaOrder[]>(stringResult);
                    
                    var ordersSortedByToppings = (from c in pizzaOrders 
                        select new
                        {
                            ToppingsList = string.Join(",", c.toppings.ToArray()),
                        })
                        .GroupBy(c => c.ToppingsList)
                        .Select(group => new
                        {
                            ToppingsList = group.Key,
                            Count = group.Count()
                        })
                        .OrderByDescending(c => c.Count)
                        .ToList();

                    int index = 0;
                    foreach (var order in ordersSortedByToppings)
                    {
                        if (index >= MAX_RECORDS_DISPLAYED)
                        {
                            break;
                        }

                        Console.WriteLine("Ordered {0} times: {1} ", order.Count, order.ToppingsList);
                        index++;
                    }
                }
                catch (HttpRequestException ex)
                {
                    Console.WriteLine(ex.ToString());
                }
            }
        }
        
        public static void Main(string[] args)
        {
            MainAsync(args).GetAwaiter().GetResult();
        }
    }
}