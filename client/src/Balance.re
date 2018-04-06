[@bs.module] external css : Js.t({..}) = "./Balance.css";

type state = {
  publicKey: string,
  loading: bool,
};

type action =
  | SetPublicKey(string)
  | BalanceFetch
  | BalanceFailedToFetch
  | BalanceFetched(int);

let component = ReasonReact.reducerComponent(__MODULE__);

let make = (_) => {
  ...component,

  initialState: () => { publicKey: "", loading: false },

  reducer: (action, state) =>
    switch(action) {
    | SetPublicKey(key) => ReasonReact.Update({ ...state, publicKey: key })
    | BalanceFetch =>
      ReasonReact.UpdateWithSideEffects(
        {...state, loading: true},
        self => {
          let publicKey = state.publicKey;
          Js.Promise.(
            Fetch({j|http://localhost:8000/api/services/wallets/v1/wallets/$(publicKey)|j})
            |> then_(Fetch.response.json)
            |> then_(field("balance", int))
            |> then_(balance => self.send(BalanceFetched(balance)))
            |> catch(_ => Js.Promise.resolve(self.send(BalanceFailedToFetch)))
          )
        }
      )
    | BalanceFailedToFetch => ReasonReact.Update({ ...state, loading: false })
    | BalanceFetched(balance) => ReasonReact.update({ balance, loading: false })
    },


  render: self =>
    <div className=css##balance>
      <input
        value=self.state.publicKey
        onChange=(e => self.send(SetPublicKey((e |> ReactEventRe.Form.target |> ReactDOMRe.domElementToObj)##value)))
      />
      <button onClick=(_ => self.send(BalanceFetch))>
        ("Fetch" |> ReasonReact.stringToElement)
      </button>
    </div>,
};

