[@bs.module] external css : Js.t({..}) = "./Balance.css";

type state = {
  publicKey: string,
  loading: bool,
};

type action =
  | SetPublicKey(string)
  | BalanceFetch
  | BalanceFailedFetchTo
  | BalalceFetched(int);

let component = ReasonReact.reducerComponent(__MODULE__);

let make = (_) => {
  ...component,

  initialState: () => { publicKey: "" },

  reducer: (action, state) =>
    switch(action) {
    | SetPublicKey(key) => ReasonReact.Update({ ...state, publicKey: key })
    | BalanceFetch => ReasonReact.SideEffects(self => {
      Js.Promise.(
        Fetch({j|http://localhost:8000/wallets/${state.publicKey}|j})
        |> then_(Fetch.response.json)
      );
      Js.log(self.state.publicKey)
    })
    },

  render: self =>
    <div className=css##balance>
      <input
        value=self.state.publicKey
        onChange=(e => self.send(SetPublicKey((e |> ReactEventRe.Form.target |> ReactDOMRe.domElementToObj)##value)))
      />
      <button onClick=(_ => self.send(FetchBalance))>
        ("Fetch" |> ReasonReact.stringToElement)
      </button>
    </div>,
};

